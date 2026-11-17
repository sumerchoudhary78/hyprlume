import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.theme
ColumnLayout {
    id:root
    property string label:""
    property string value:""
    property string controlName:""
    property int limit:512
    signal edited(string value)
    spacing:6
    Text {text:root.label; color:Theme.inkDim; font.family:Theme.fontSans; font.pixelSize:12}
    TextField {
        id:input
        objectName:root.controlName
        Layout.fillWidth:true; Layout.preferredHeight:39
        maximumLength:root.limit; selectByMouse:true
        color:Theme.ink; font.family:Theme.fontSans; font.pixelSize:13
        background:Rectangle {radius:12; color:"#80102136"; border.color:parent.activeFocus ? Theme.cyan : Theme.rim}
        onTextEdited:root.edited(text)
        Component.onCompleted:text=root.value
        Binding {target:input;property:"text";value:root.value;when:!input.activeFocus;restoreMode:Binding.RestoreNone}
    }
}
