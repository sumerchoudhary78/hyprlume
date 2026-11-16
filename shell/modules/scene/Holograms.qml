import QtQuick
import QtQuick3D
import QtQuick3D.Helpers
import "../../assets/data/WorldMap.js" as WorldMap
import "Geometry.js" as Geometry

// One small, on-demand 3D viewport. No animation loop, imported model pack,
// network access, shadows or full-screen postprocessing.
Item {
    id: root
    property real globeAngle: -28
    signal globeActivated()
    signal cityActivated()

    readonly property var coastVertices: Geometry.coasts(WorldMap.coastlines)
    readonly property var landVertices: Geometry.land(WorldMap.landPoints)
    readonly property var gridVertices: Geometry.grid()
    readonly property var cityMesh: Geometry.city()

    View3D {
        anchors.fill: parent
        environment: SceneEnvironment {
            backgroundMode: SceneEnvironment.Transparent
            antialiasingMode: SceneEnvironment.MSAA
            antialiasingQuality: SceneEnvironment.High
            tonemapMode: SceneEnvironment.TonemapModeNone
        }
        OrthographicCamera {
            id: camera
            position: Qt.vector3d(0, 240, 640)
            eulerRotation.x: -19
            horizontalMagnification: 1
            verticalMagnification: 1
        }
        camera: camera
        DirectionalLight { eulerRotation: Qt.vector3d(-40, -35, 0); brightness: 1.2; color: "#9bceff" }
        PrincipledMaterial { id: wire; lighting: PrincipledMaterial.NoLighting; baseColor: "#72d5ff"; emissiveFactor: Qt.vector3d(0.15, 0.35, 0.6) }
        PrincipledMaterial { id: fineWire; lighting: PrincipledMaterial.NoLighting; baseColor: "#18375a" }
        PrincipledMaterial { id: violetWire; lighting: PrincipledMaterial.NoLighting; baseColor: "#ad88ff"; emissiveFactor: Qt.vector3d(0.25, 0.15, 0.5) }
        Node {
            x: -335; y: 22
            Node {
                y: 60
                eulerRotation.y: root.globeAngle
                Model {
                    source: "#Sphere"
                    scale: Qt.vector3d(1.1, 1.1, 1.1)
                    materials: PrincipledMaterial {
                        baseColor: "#14325e"
                        emissiveFactor: Qt.vector3d(0.015, 0.04, 0.09)
                        metalness: 0.3
                        roughness: 0.3
                    }
                }
                Model {
                    geometry: ProceduralMesh { primitiveMode: ProceduralMesh.Lines; positions: root.gridVertices }
                    materials: fineWire
                }
                Model {
                    geometry: ProceduralMesh { primitiveMode: ProceduralMesh.Lines; positions: root.coastVertices }
                    materials: wire
                }
                Model {
                    geometry: ProceduralMesh { primitiveMode: ProceduralMesh.Points; positions: root.landVertices }
                    materials: PrincipledMaterial {
                        lighting: PrincipledMaterial.NoLighting
                        baseColor: "#b0d8ff"
                        pointSize: 1.25
                        emissiveFactor: Qt.vector3d(0.12, 0.2, 0.35)
                    }
                }
                Model {
                    eulerRotation: Qt.vector3d(24, 0, -15)
                    geometry: TorusGeometry { radius: 72; tubeRadius: 0.45; rings: 96; segments: 6 }
                    materials: violetWire
                }
            }
            Model {
                y: -2
                source: "#Cylinder"
                scale: Qt.vector3d(1.2, 0.025, 1.2)
                materials: PrincipledMaterial { baseColor: "#061b36"; metalness: 0.6; roughness: 0.2 }
            }
            Repeater3D {
                model: 3
                Model {
                    required property int index
                    y: index * 2
                    geometry: TorusGeometry { radius: 57 + index * 10; tubeRadius: 0.6; rings: 80; segments: 6 }
                    materials: index === 1 ? violetWire : wire
                }
            }
        }
        Node {
            x: 340; y: 12
            eulerRotation.y: -24
            Model {
                source: "#Cube"
                y: -5
                scale: Qt.vector3d(2.03, 0.08, 1.25)
                materials: PrincipledMaterial { lighting: PrincipledMaterial.NoLighting; baseColor: "#092a42" }
            }
            Model {
                geometry: ProceduralMesh { positions: root.cityMesh.solid; colors: root.cityMesh.colors }
                materials: PrincipledMaterial { lighting: PrincipledMaterial.NoLighting; baseColor: "white"; cullMode: Material.NoCulling }
            }
            Model {
                geometry: ProceduralMesh { primitiveMode: ProceduralMesh.Lines; positions: root.cityMesh.wire }
                materials: wire
            }
        }
    }
    MouseArea {
        objectName: "hologramGlobeRegion"
        x: 25; y: 0; width: 200; height: parent.height
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onPositionChanged: mouse => root.globeAngle = -28 + (mouse.x / width - 0.5) * 45
        onClicked: root.globeActivated()
        Accessible.name: "Explore system network"
    }
    MouseArea {
        objectName: "hologramCityRegion"
        anchors.right: parent.right; width: 220; height: parent.height
        cursorShape: Qt.PointingHandCursor
        onClicked: root.cityActivated()
        Accessible.name: "Show container activity"
    }
}
