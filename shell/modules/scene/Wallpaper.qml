import QtQuick

Item {
    readonly property bool ready: image.status === Image.Ready
    Image {
        id: image
        anchors.fill: parent
        source: "../../assets/backgrounds/observatory.png"
        fillMode: Image.PreserveAspectCrop
        asynchronous: true
        cache: true
        smooth: true
    }
    Rectangle {
        anchors.fill: parent
        gradient: Gradient {
            GradientStop { position: 0; color: "#09040a16" }
            GradientStop { position: 0.5; color: "#12020a19" }
            GradientStop { position: 1; color: "#30020711" }
        }
    }
}
