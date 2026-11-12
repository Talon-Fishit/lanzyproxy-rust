//! The command table.
//!
//! Every command lives here exactly once, so the registry cannot drift and
//! the `/proxy` list is a straight walk over one array.

use crate::Context;

/// One entry of the command table.
pub struct Entry {
    pub name: &'static str,
    pub summary: &'static str,
    pub keywords: &'static str,
    pub handler: fn(&mut Context, &str),
}

/// Every registered command.
pub static ALL: &[Entry] = &[
    // -- utility ----------------------------------------------------------
    Entry { name: "proxy", summary: "Proxy Commands List", keywords: "cmd,commands,list,help,info,all,feature,proxy", handler: crate::utility::proxy },
    Entry { name: "settings", summary: "Proxy Settings", keywords: "settings,config,load,save,slot,reset,delete,feature,main,setting", handler: crate::utility::settings },
    Entry { name: "options", summary: "Fast Mag/Gaia/UT Retrieve", keywords: "mag,gaia,retrieve,ut", handler: crate::utility::options },
    Entry { name: "keep", summary: "Save Current Proxy Settings", keywords: "save,keep,settings,command,all,current,enabled,proxy,setting", handler: crate::utility::keep },
    Entry { name: "reset", summary: "Reset all /pos for autohost", keywords: "reset,autohost,pos,tp,win1,win2,win3,win4,all,host", handler: crate::utility::reset },
    Entry { name: "hotkey", summary: "Custom Hotkeys", keywords: "hotkey,key,keyboard,command,list,custom,none", handler: crate::utility::hotkey },
    Entry { name: "logs", summary: "Proxy Logs Page", keywords: "logs,history,console,drop,join,trade,world,collected,in,saved,with", handler: crate::utility::logs },
    Entry { name: "server", summary: "Show Growtopia Server Info", keywords: "server,info,address,meta,growtopia", handler: crate::utility::server },
    Entry { name: "logoff", summary: "Fast Log Off From Growtopia", keywords: "logoff,logout,quit,account,log,off,disconnect,out", handler: crate::utility::logoff },
    Entry { name: "calc", summary: "Calculates & Solves Math Equations", keywords: "math,calculate,solve,*,+,/,=,calc,number,divide,equal,equation,sum", handler: crate::utility::calc },
    Entry { name: "switch", summary: "Fast Account Switch Page", keywords: "account,switch,login,growid,save,password", handler: crate::utility::switch },
    Entry { name: "status", summary: "Proxy Status Overview", keywords: "status,info,connection", handler: crate::utility::status },
    // -- automation -------------------------------------------------------
    Entry { name: "autofarm", summary: "Auto Farm Page", keywords: "auto,farm,punch,block,interval,chand,ms,pepper,build,place,select", handler: crate::automation::autofarm },
    Entry { name: "farm", summary: "Enable AutoFarm Shortcut", keywords: "auto,farm,shortcut,start,block,ms,punch,build,enable,place,select", handler: crate::automation::farm },
    Entry { name: "autoplant", summary: "Auto Plant Page", keywords: "auto,plant,seed,grow,ms,start,tree,area,available,select,splice", handler: crate::automation::autoplant },
    Entry { name: "plant", summary: "Enable Auto Plant Shortcut", keywords: "plant,seed,shortcut,auto,farm,grow,ms,start,tree,area,available,enable,select,splice", handler: crate::automation::plant },
    Entry { name: "autoharvest", summary: "Auto Harvest Page", keywords: "auto,harvest,tree,fruit,drop,farm,items,ms", handler: crate::automation::autoharvest },
    Entry { name: "harvest", summary: "Enable Auto Harvest Shortcut", keywords: "harvest,tree,fruit,shortcut,auto,drop,farm,items,ms,start,available,enable", handler: crate::automation::harvest },
    Entry { name: "autocollect", summary: "Autocollect Page", keywords: "collect,items,floating,fast,giveaway,take", handler: crate::automation::autocollect },
    Entry { name: "collect", summary: "Fast Collect Floating Items Once", keywords: "collect,items,once,fast,floating,giveaway,shortcut,take", handler: crate::automation::collect },
    Entry { name: "autocook", summary: "Auto Cook Page", keywords: "cook,food,star,oven,cooking,auto,autocook,ingredients,toss", handler: crate::automation::autocook },
    Entry { name: "startcook", summary: "Start Auto Cooking", keywords: "cook,start,begin,auto,autocook,food,oven,star,ingredient,toss", handler: crate::automation::startcook },
    Entry { name: "fish", summary: "Toggle Shortcut Autofish", keywords: "fish,bait,rod,autofish,auto,lb,shortcut,toggle,wiggly,water,worm", handler: crate::automation::fish },
    Entry { name: "providers", summary: "Enable Auto Harvest Providers Shortcut", keywords: "provider,chicken,cow,atm,auto,collect,farm,harvest,produc,punch,shortcut,enable,provide", handler: crate::automation::providers },
    Entry { name: "crime", summary: "Enables Auto Crime Shortcut", keywords: "crime,card,wave,against,auto,shortcut,fight", handler: crate::automation::crime },
    Entry { name: "acc", summary: "Enable Auto Accept Access", keywords: "accept,access,lock,acess,shortcut,wl,to", handler: crate::automation::acc },
    Entry { name: "autolock", summary: "Toggle Auto Wrench Lock Shortcut", keywords: "lock,auto,wrench,apply,public,shortcut,big,empty,huge,small", handler: crate::automation::autolock },
    Entry { name: "auto", summary: "Auto Ban/Pull On Enter World Page", keywords: "enter,join,ban,pull,auto,autoban,autobanner,growid,world,all,any,player,specific", handler: crate::automation::auto },
    Entry { name: "anti", summary: "Anti Effects Page (Gravity, Punch, Lava)", keywords: "anti,punch,gravity,lava,antilava,antipunch,antigravity,jammer,nuke", handler: crate::automation::anti },
    // -- economy ----------------------------------------------------------
    Entry { name: "vend", summary: "Vending Page", keywords: "vend,buy,restock,stock,wls,fast,items,purchase,take,withdraw", handler: crate::economy::vend },
    Entry { name: "fastvend", summary: "Toggle Fast Vend Shortcut", keywords: "vend,buy,fast,shortcut,purchase", handler: crate::economy::fastvend },
    Entry { name: "shop", summary: "Fast Auto Purchase From Shop", keywords: "shop,buy,bulk,gems,fast,ssp,growtopia,purchase", handler: crate::economy::shop },
    Entry { name: "dice", summary: "Fast Roll Dice", keywords: "dice,roll,random,fast,number,speed,instant,show", handler: crate::economy::dice },
    Entry { name: "check", summary: "Display Account Untradeable Items & Level", keywords: "account,level,quest,untrade,info,display,role,tradeable", handler: crate::economy::check },
    Entry { name: "gazette", summary: "Real Growtopia News Gazette", keywords: "news,gazette,growtopia,real", handler: crate::economy::gazette },
    // -- visuals ----------------------------------------------------------
    Entry { name: "clothes", summary: "Visual Clothes Equip", keywords: "clothes,visual,equip,rayman,wings,find,item,legendary,put on,clothe,fake,proxy", handler: crate::visuals::clothes },
    Entry { name: "unequip", summary: "Unequips All Visual Clothes", keywords: "unequip,visual,off,clothes,equip,naked,take", handler: crate::visuals::unequip },
    Entry { name: "titles", summary: "Visual Titles Page", keywords: "title,legend,g4g,doctor,dr.,flag,custom,name,of", handler: crate::visuals::titles },
    Entry { name: "weather", summary: "Visual Weather Page", keywords: "weather,visual,machine,background,custom", handler: crate::visuals::weather },
    Entry { name: "country", summary: "Country Flag Changer", keywords: "country,flag,countries,nepal,change,custom", handler: crate::visuals::country },
    Entry { name: "spirit", summary: "Show Spirit Board Info", keywords: "spirit,board,ghost,info,boss,clothing,dark", handler: crate::visuals::spirit },
    Entry { name: "ui", summary: "Change UI Colors Page", keywords: "ui,color,theme,any,change,colour,custom,select,text", handler: crate::visuals::ui },
    Entry { name: "speed", summary: "Custom Player Speed Page", keywords: "speed,jump,gravity,fly,float,fast,change,custom,fall,player,slow,swim,water", handler: crate::visuals::speed },
    // -- navigation -------------------------------------------------------
    Entry { name: "pathfind", summary: "Pathfinder Settings Page", keywords: "pathfind,delay,speed,pathfinder,pathfinding,blocks,teleport,tp,setting,travel", handler: crate::navigation::pathfind },
    Entry { name: "pfind", summary: "Pathfind To Nearest Item", keywords: "pathfind,item,nearest,pfind,collect,teleport,tp,closest", handler: crate::navigation::pfind },
    Entry { name: "pvend", summary: "Pathfind To Cheapest Vending Machine", keywords: "pathfind,vend,cheapest,pvend,pathfinder,pathfinding,find,item,buy,cheap,purchase,sell", handler: crate::navigation::pvend },
    Entry { name: "find", summary: "Find Item In Tracked Drops", keywords: "find,item,drop,search,clothes,item id,seeds,visual,add,finder,id,inventory,itemid,rarity,results", handler: crate::navigation::find },
    Entry { name: "render", summary: "Render World Map To PNG", keywords: "render,map,png,image,world,full,generate,picture", handler: crate::navigation::render },
    Entry { name: "parkour", summary: "Fast Complete Parkour World", keywords: "parkour,wolf,clash,pathfind,world,complete,finish", handler: crate::navigation::parkour },
    Entry { name: "blocks", summary: "Show Growscan 9000 Blocks Shortcut", keywords: "blocks,growscan,gs,9000,shortcut,world", handler: crate::navigation::blocks },
    Entry { name: "gs", summary: "Free Growscan 9000", keywords: "growscan,items,blocks,free,9000,floating,fruits,gs,tree,world,show,stats,view", handler: crate::navigation::gs },
    Entry { name: "items", summary: "Show Growscan Floating Items Shortcut", keywords: "growscan,floating,items,9000,gs,item,shortcut,world,gem", handler: crate::navigation::items },
    Entry { name: "tree", summary: "Show Fully Grown Planted Seeds Fruits", keywords: "tree,fruit,seed,grow,number,plant,amount,fully,show", handler: crate::navigation::tree },
    Entry { name: "chest", summary: "Show Items Inside Chest", keywords: "chest,items,hidden,closed,inside,show", handler: crate::navigation::chest },
    // -- world tools ------------------------------------------------------
    Entry { name: "world", summary: "World Options & Average People Count", keywords: "world,trade,unban,info,number,amount,average,based,count,data,people,player", handler: crate::worldtools::world },
    Entry { name: "worlds", summary: "List of Worlds Owned by You", keywords: "worlds,own,wl,list,teleport,dl,world,name,you", handler: crate::worldtools::worlds },
    Entry { name: "list", summary: "Saved Worlds Page", keywords: "save,worlds,warp,list,addcommandaswarpworld,all,doorid,name", handler: crate::worldtools::list_worlds },
    Entry { name: "scan", summary: "Enable Free Extract-O-Snap", keywords: "extract,snap,collect,items,-o-,1,floating,free,punch,check,only,scan,take,tile", handler: crate::worldtools::scan },
    // -- moderation -------------------------------------------------------
    Entry { name: "wrench", summary: "Fast Wrench Mode", keywords: "wrench,mode,pull,kick,ban,door,fast,click,close,entrance,open,right", handler: crate::moderation::wrench },
    Entry { name: "locked", summary: "Show Locked Doors Label Text", keywords: "lock,door,label,hidden,public,closed,private,red,show,text", handler: crate::moderation::locked },
    Entry { name: "lost", summary: "Fast Small & Big & Huge Locks Apply", keywords: "lock,apply,public,big,empty,huge,ignore,open,small,space", handler: crate::moderation::lost },
    Entry { name: "moddetect", summary: "Toggle Mod Detect Shortcut", keywords: "mod,detect,toggle,shortcut", handler: crate::moderation::moddetect },
    Entry { name: "track", summary: "Shows Dropped/Collected Items By Others", keywords: "track,drop,collect,console,item,live,amount,other,player", handler: crate::moderation::track },
    Entry { name: "pkickall", summary: "Fast Kick All", keywords: "kick,all,players,respawn,die,people", handler: crate::moderation::pkickall },
    Entry { name: "hidden", summary: "Show Detected Hidden Moderators", keywords: "mod,hidden,detect,list,@,with proxy,ago", handler: crate::moderation::hidden },
    // -- casino -----------------------------------------------------------
    Entry { name: "roulette", summary: "Fast Roulette, Logs & Real/Fake Spin", keywords: "roulette,spin,wheel,casino,fast,reme,qeme,leme,roulete,spining,spinning", handler: crate::casino::roulette },
    Entry { name: "showspin", summary: "Show Last Roulette Spin Per Player", keywords: "roulette,spin,last,recent,number,showspin", handler: crate::casino::showspin },
];

/// How many commands are registered.
pub fn count() -> usize {
    ALL.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn every_command_has_a_unique_name() {
        let mut seen = HashSet::new();
        for cmd in ALL {
            assert!(seen.insert(cmd.name), "duplicate command {}", cmd.name);
        }
    }

    #[test]
    fn the_table_is_not_empty() {
        assert_eq!(count(), 67);
    }
}
