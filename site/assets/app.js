const viewData = {
  desktop: {
    image: "desktop",
    alt: "The Hyprlume desktop with glass panels, live metrics and a holographic action deck",
    caption: "Desktop · Quickshell + Qt Quick 3D",
    guide: "docs/homescreen.md",
  },
  connections: {
    image: "connections",
    alt: "The Connections overlay with Wi-Fi networks, Bluetooth devices and radio controls",
    caption: "Connections · NetworkManager + BlueZ",
    guide: "docs/connectivity.md",
  },
};

function selectTab(tab, tabs) {
  for (const item of tabs) {
    const active = item === tab;
    item.setAttribute("aria-selected", String(active));
    item.tabIndex = active ? 0 : -1;
    document.getElementById(item.getAttribute("aria-controls")).hidden =
      !active;
  }
  const view = viewData[tab.id.replace("tab-", "")];
  if (view) {
    const image = document.getElementById("explorer-image");
    image.srcset = `assets/media/${view.image}-800.webp 800w, assets/media/${view.image}-1600.webp 1600w`;
    image.src = `assets/media/${view.image}-1600.webp`;
    image.alt = view.alt;
    document.getElementById("explorer-caption").textContent = view.caption;
    document
      .querySelector("[data-expand-image]")
      .setAttribute("aria-label", `View full screen: ${view.image} screenshot`);
    document.querySelector(".explorer-guide").href =
      `https://github.com/sumerchoudhary78/hyprlume/blob/main/${view.guide}`;
  }
}

for (const group of document.querySelectorAll('[role="tablist"]')) {
  const tabs = [...group.querySelectorAll('[role="tab"]')];
  group.addEventListener("click", (event) => {
    const tab = event.target.closest('[role="tab"]');
    if (tabs.includes(tab)) selectTab(tab, tabs);
  });
  group.addEventListener("keydown", (event) => {
    const index = tabs.indexOf(document.activeElement);
    if (index === -1) return;
    const offsets = { ArrowRight: 1, ArrowLeft: -1 };
    let next;
    if (event.key in offsets)
      next = (index + offsets[event.key] + tabs.length) % tabs.length;
    else if (event.key === "Home") next = 0;
    else if (event.key === "End") next = tabs.length - 1;
    else return;
    event.preventDefault();
    selectTab(tabs[next], tabs);
    tabs[next].focus();
  });
}

const examples = {
  workspace: ["“Switch to workspace two.”", "Switch to workspace 2"],
  terminal: ["“Open terminal.”", "Open configured terminal"],
  volume: ["“Set the volume to thirty-five percent.”", "Set volume to 35%"],
};
for (const button of document.querySelectorAll("[data-command]")) {
  button.addEventListener("click", () => {
    for (const item of document.querySelectorAll("[data-command]")) {
      item.setAttribute("aria-pressed", String(item === button));
    }
    const [transcript, plan] = examples[button.dataset.command];
    document.getElementById("demo-transcript").textContent = transcript;
    document.getElementById("demo-plan").textContent = plan;
  });
}

let toastTimeout;
for (const button of document.querySelectorAll("[data-copy]")) {
  button.addEventListener("click", async () => {
    const code = document.getElementById(button.dataset.copy);
    const status = document.getElementById("copy-status");
    clearTimeout(toastTimeout);
    try {
      await navigator.clipboard.writeText(code.textContent);
      status.textContent = "Command copied.";
    } catch {
      const range = document.createRange();
      range.selectNodeContents(code);
      const selection = window.getSelection();
      selection.removeAllRanges();
      selection.addRange(range);
      status.textContent = "Command selected. Press Ctrl+C or ⌘C to copy.";
    }
    status.classList.add("is-visible");
    toastTimeout = setTimeout(
      () => status.classList.remove("is-visible"),
      3500,
    );
  });
}

const menu = document.querySelector(".menu-toggle");
const nav = document.getElementById("site-nav");
function closeMenu() {
  menu.setAttribute("aria-expanded", "false");
  menu.setAttribute("aria-label", "Open navigation");
  nav.classList.remove("is-open");
}
menu.addEventListener("click", () => {
  const open = menu.getAttribute("aria-expanded") !== "true";
  menu.setAttribute("aria-expanded", String(open));
  menu.setAttribute(
    "aria-label",
    open ? "Close navigation" : "Open navigation",
  );
  nav.classList.toggle("is-open", open);
});
nav.addEventListener("click", (event) => {
  if (event.target.closest("a")) closeMenu();
});
document.addEventListener("keydown", (event) => {
  if (event.key === "Escape" && menu.getAttribute("aria-expanded") === "true") {
    closeMenu();
    menu.focus();
  }
});

const dialog = document.getElementById("screenshot-dialog");
const expandButton = document.querySelector("[data-expand-image]");
expandButton.addEventListener("click", () => {
  const image = document.getElementById("explorer-image");
  const expanded = document.getElementById("dialog-image");
  expanded.src = image.src;
  expanded.alt = image.alt + "; sample data";
  document.getElementById("dialog-caption").textContent =
    document.getElementById("explorer-caption").textContent + " · sample data";
  dialog.showModal();
  document.body.classList.add("dialog-open");
});
dialog
  .querySelector(".dialog-close")
  .addEventListener("click", () => dialog.close());
dialog.addEventListener("click", (event) => {
  if (event.target === dialog) dialog.close();
});
dialog.addEventListener("close", () => {
  document.body.classList.remove("dialog-open");
  expandButton.focus();
});
