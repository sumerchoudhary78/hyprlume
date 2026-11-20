import QtQuick
import QtQuick.Controls

Flickable {
    id: root
    clip: true
    interactive: contentHeight > height
    boundsBehavior: Flickable.StopAtBounds
    ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }

    // Keyboard users can reach rows below the visible area on short outputs.
    Connections {
        target: root.Window.window
        function onActiveFocusItemChanged() {
            if (!root.interactive)
                return;
            const item = root.Window.window?.activeFocusItem;
            for (let parent = item; parent; parent = parent.parent) {
                if (parent !== root.contentItem)
                    continue;
                const top = item.mapToItem(root.contentItem, 0, 0).y;
                const bottom = top + item.height;
                if (top < root.contentY)
                    root.contentY = Math.max(0, top);
                else if (bottom > root.contentY + root.height)
                    root.contentY = Math.min(root.contentHeight - root.height, bottom - root.height);
                break;
            }
        }
    }
}
