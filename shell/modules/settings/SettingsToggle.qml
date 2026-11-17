import QtQuick
import QtQuick.Layouts
import qs.components
import qs.theme
RowLayout {
    id:root
    property string label:""
    property string description:""
    property bool checked:false
    property string controlName:""
    signal toggled(bool checked)
    spacing:14
    ColumnLayout {
        Layout.fillWidth:true; spacing:4
        Text {Layout.fillWidth:true; text:root.label; color:Theme.ink; font.family:Theme.fontSans; font.pixelSize:14; wrapMode:Text.Wrap}
        Text {Layout.fillWidth:true; text:root.description; visible:text.length>0; color:Theme.inkFaint; font.family:Theme.fontSans; font.pixelSize:11; wrapMode:Text.Wrap}
    }
    GlassCheckBox {objectName:root.controlName; checked:root.checked; Accessible.name:root.label; onToggled:root.toggled(checked)}
}
