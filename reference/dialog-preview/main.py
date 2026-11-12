#!/usr/bin/env python3
import argparse
import html as html_module
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
CHROME_CANDIDATES = [
    "chromium",
    "chromium-browser",
    "google-chrome",
    "google-chrome-stable",
    "/usr/bin/chromium",
    "/usr/bin/chromium-browser",
    "/usr/bin/google-chrome",
]

PROBE = """
<script>
(function () {
  var steps = [];
  function q(s) { return document.querySelector(s); }
  function qa(s) { return document.querySelectorAll(s); }
  try {
    var picker = q('#picker');
    // Read the dialogs from the page itself rather than a hardcoded list, so
    // the test keeps working when a sample dialog is added or removed.
    var ids = picker ? Array.prototype.map.call(picker.options, function (o) { return o.value; }) : [];
    var okMove = !!picker && ids.length > 0;
    var rowsMove = 0;
    if (picker) {
      for (var i = 0; i < ids.length; i++) {
        picker.value = ids[i];
        picker.dispatchEvent(new Event('change'));
        rowsMove = qa('#render .gt-row').length;
        if (rowsMove === 0) okMove = false;
      }
    }
    steps.push({ step: 'move', passed: !!okMove, detail: 'picker switched ' + ids.length + ' dialogs, each rendered rows' });

    var t = q('#markup');
    var nl = String.fromCharCode(10), bt = String.fromCharCode(96);
    t.value = 'add_label|' + bt + '2green ' + bt + '4red ' + bt + '9blue ' + bt + 'wwhite ' + bt + '#purple ' + bt + bt + 'reset|0|100|' + nl +
      'add_button|b|' + bt + 'wClick|0|123|5|' + nl +
      'end_dialog|probe|OK|Cancel|';
    t.dispatchEvent(new Event('input'));
    var spans = qa('#render .gt-body span[style]').length;
    var btn = q('#render .gt-button');
    var badge = q('#render .item-count');
    steps.push({ step: 'act', passed: spans >= 5 && !!btn && !!badge, detail: 'edited markup; ' + spans + ' colour spans, button and item badge rendered' });

    var title = q('#render .gt-title');
    var buttons = qa('#render .gt-button').length;
    var labels = qa('#render .gt-label').length;
    steps.push({ step: 'hud', passed: !!title && buttons >= 3 && labels >= 1, detail: 'title "' + (title ? title.textContent : '') + '", ' + labels + ' labels, ' + buttons + ' buttons drawn' });
  } catch (e) {
    steps.push({ step: 'error', passed: false, detail: String(e) });
  }
  var pre = document.createElement('pre');
  pre.id = 'selftest-out';
  pre.textContent = JSON.stringify({ version: 1, steps: steps });
  document.body.appendChild(pre);
})();
</script>
"""


def find_chrome():
    for name in CHROME_CANDIDATES:
        path = shutil.which(name) if not name.startswith("/") else (name if os.path.exists(name) else None)
        if path:
            return path
    return None


def base_cmd(chrome, extra):
    return [chrome, "--headless", "--no-sandbox", "--disable-gpu"] + extra


def run_screenshot(chrome, out_path):
    out = os.path.abspath(out_path)
    os.makedirs(os.path.dirname(out) or ".", exist_ok=True)
    tmp = out + ".tmp.png"
    url = "file://" + os.path.join(HERE, "index.html")
    cmd = base_cmd(chrome, ["--hide-scrollbars", "--window-size=1440,900", "--screenshot=" + tmp, url])
    try:
        subprocess.run(cmd, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=90)
    except (FileNotFoundError, subprocess.TimeoutExpired, subprocess.CalledProcessError) as exc:
        print("screenshot failed: %s" % exc, file=sys.stderr)
        return 1
    if not os.path.exists(tmp) or os.path.getsize(tmp) == 0:
        print("screenshot was not produced", file=sys.stderr)
        return 1
    os.replace(tmp, out)
    return 0


def run_self_test(chrome, out_path):
    src = os.path.join(HERE, "index.html")
    with open(src, "r", encoding="utf-8") as f:
        page = f.read()
    if "</body>" not in page:
        print("index.html has no </body> to attach the probe", file=sys.stderr)
        return 1
    copy = page.replace("</body>", PROBE + "</body>")
    fd, tmp = tempfile.mkstemp(suffix=".html", prefix="dialog-selftest-")
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as f:
            f.write(copy)
        cmd = base_cmd(chrome, ["--virtual-time-budget=5000", "--dump-dom", "file://" + tmp])
        try:
            proc = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, timeout=90)
        except (FileNotFoundError, subprocess.TimeoutExpired) as exc:
            print("self-test browser failed: %s" % exc, file=sys.stderr)
            return 1
        dom = proc.stdout.decode("utf-8", "replace")
    finally:
        try:
            os.unlink(tmp)
        except OSError:
            pass

    m = re.search(r'<pre id="selftest-out">(.*?)</pre>', dom, re.S)
    if not m:
        print("self-test probe did not report", file=sys.stderr)
        return 1
    text = html_module.unescape(m.group(1)).strip()
    try:
        data = json.loads(text)
    except json.JSONDecodeError as exc:
        print("self-test report is not JSON: %s" % exc, file=sys.stderr)
        return 1

    out = os.path.abspath(out_path)
    os.makedirs(os.path.dirname(out) or ".", exist_ok=True)
    with open(out, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2)
        f.write("\n")
    return 0


def main():
    parser = argparse.ArgumentParser(description="Growtopia dialog preview launcher")
    parser.add_argument("--screenshot", metavar="FILE", help="save a PNG of the page and exit")
    parser.add_argument("--self-test", metavar="FILE", help="drive the page and save a JSON report")
    args = parser.parse_args()

    if not args.screenshot and not args.self_test:
        print("usage: main.py --screenshot FILE.png | --self-test FILE.json")
        return 0

    chrome = find_chrome()
    if not chrome:
        print("no headless browser found (looked for chromium)", file=sys.stderr)
        return 1

    status = 0
    if args.screenshot:
        status = run_screenshot(chrome, args.screenshot) or status
    if args.self_test:
        status = run_self_test(chrome, args.self_test) or status
    return status


if __name__ == "__main__":
    sys.exit(main())
