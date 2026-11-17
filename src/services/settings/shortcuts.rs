//! Validated declarations only. We never edit or reload the running compositor.
use crate::config::Shortcuts;
fn actions(shortcuts: &Shortcuts) -> [(&str, &str); 4] {
    [
        (&shortcuts.launcher, "launcher"),
        (&shortcuts.windows, "launcher --windows"),
        (&shortcuts.tools, "clipboard"),
        (&shortcuts.lock, "lock"),
    ]
}
pub(super) fn hyprlang(shortcuts: &Shortcuts) -> String {
    actions(shortcuts)
        .iter()
        .map(|(binding, command)| {
            let (modifiers, key) = binding.rsplit_once('+').unwrap();
            format!(
                "bind = {}, {}, exec, hyprlume {command}\n",
                modifiers.replace('+', " "),
                key
            )
        })
        .collect()
}
pub(super) fn lua(shortcuts: &Shortcuts) -> String {
    actions(shortcuts)
        .iter()
        .map(|(binding, command)| {
            format!(
                "hl.bind(\"{}\", hl.dsp.exec_cmd(\"hyprlume {command}\"))\n",
                binding.replace('+', " + ")
            )
        })
        .collect()
}
