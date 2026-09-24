(function () {
  var STORAGE_KEY = "theme";

  function currentTheme() {
    try {
      return localStorage.getItem(STORAGE_KEY) || "dark";
    } catch (_) {
      return "dark";
    }
  }

  function applyTheme(theme) {
    var root = document.documentElement;
    if (theme === "dark") {
      root.classList.add("dark");
      root.classList.remove("light");
    } else {
      root.classList.remove("dark");
      root.classList.add("light");
    }
    try {
      localStorage.setItem(STORAGE_KEY, theme);
    } catch (_) {}
  }

  // Initial theme is applied by an inline script in base.html to avoid a flash;
  // this keeps the storage/class state in sync for direct DOM reads.
  applyTheme(currentTheme());

  document.querySelectorAll("[data-theme-toggle]").forEach(function (button) {
    button.addEventListener("click", function () {
      var next = currentTheme() === "dark" ? "light" : "dark";
      applyTheme(next);
    });
  });

  function openModal(dialog) {
    if (!dialog) return;
    try {
      if (typeof dialog.showModal === "function") {
        dialog.showModal();
      } else {
        dialog.setAttribute("open", "");
      }
    } catch (_) {
      dialog.setAttribute("open", "");
    }
  }

  function closeModal(dialog) {
    if (!dialog) return;
    try {
      if (typeof dialog.close === "function") {
        dialog.close();
      } else {
        dialog.removeAttribute("open");
      }
    } catch (_) {
      dialog.removeAttribute("open");
    }
  }

  document.querySelectorAll("[data-modal-open]").forEach(function (button) {
    button.addEventListener("click", function () {
      openModal(document.getElementById(button.dataset.modalOpen));
    });
  });

  document.querySelectorAll("dialog[data-modal], dialog.modal").forEach(function (dialog) {
    dialog.addEventListener("click", function (event) {
      if (event.target === dialog) {
        closeModal(dialog);
      }
    });

    dialog.querySelectorAll("[data-modal-close]").forEach(function (button) {
      button.addEventListener("click", function () {
        closeModal(dialog);
      });
    });
  });
})();
