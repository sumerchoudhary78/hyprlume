import QtQuick

Item {
    id: root
    required property var backend
    property bool reducedEffects: false
    property real availableHeight: 800
    readonly property var notices: !backend.online || backend.notifications.panel_open || backend.notifications.dnd ? [] : backend.notifications.items.filter(notice => notice.popup && notice.active).slice(0, Math.max(1, Math.min(3, Math.floor(availableHeight / 280))))
    implicitHeight: stack.implicitHeight
    Column {
        id: stack
        width: parent.width; spacing: 12
        Repeater {
            model: root.notices
            NoticeCard {
                required property var modelData
                width: stack.width; height: implicitHeight
                notice: modelData; popupView: true; reducedEffects: root.reducedEffects
                onRequest: action => root.backend.send({op:"notifications",request:action})
            }
        }
    }
}
