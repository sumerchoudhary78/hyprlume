import QtQuick
import qs.services

LauncherController {
    apps: [
        {id:"org.example.Terminal.desktop",title:"Terminal",subtitle:"Command line workspace",icon:"utilities-terminal",search:"shell kitty console"},
        {id:"org.example.Code.desktop",title:"Code",subtitle:"Code editor",icon:"text-x-script",search:"development rust editor"},
        {id:"org.example.Firefox.desktop",title:"Firefox",subtitle:"Web browser",icon:"globe",search:"internet browser"}
    ]
    windows: [
        {id:"0xa",title:"Hyprlume — editor",subtitle:"Code · Workspace 2",icon:"",search:"code rust"},
        {id:"0xb",title:"Documentation",subtitle:"Firefox · Workspace 3",icon:"",search:"firefox browser"}
    ]
    onRequest: operation => { console.log("HYPRLUME_QA " + JSON.stringify({kind:"request",operation:operation})); finish(true, ""); }
}
