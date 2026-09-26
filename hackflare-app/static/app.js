(function () {
  var STORAGE_KEY = "theme";

  function currentTheme() {
    try {
      return localStorage.getItem(STORAGE_KEY) || "light";
    } catch (_) {
      return "light";
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

  // Marketing nav: the links collapse behind a menu button under 860px.
  document.querySelectorAll("[data-nav-toggle]").forEach(function (button) {
    var nav = document.getElementById(button.getAttribute("aria-controls"));
    if (!nav) return;
    button.addEventListener("click", function () {
      var open = nav.classList.toggle("is-open");
      button.setAttribute("aria-expanded", open ? "true" : "false");
    });
  });

  // --- Zone records workspace -------------------------------------------

  // Copy nameservers to the clipboard, with a brief confirmation on the chip.
  document.querySelectorAll("[data-copy]").forEach(function (button) {
    var label = button.querySelector(".lucide-icon");
    button.addEventListener("click", function () {
      var text = button.dataset.copy;
      var done = function () {
        button.classList.add("is-copied");
        setTimeout(function () {
          button.classList.remove("is-copied");
        }, 1200);
      };
      if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(text).then(done, function () {});
      } else {
        var field = document.createElement("textarea");
        field.value = text;
        document.body.appendChild(field);
        field.select();
        try {
          document.execCommand("copy");
          done();
        } catch (_) {}
        document.body.removeChild(field);
      }
    });
    if (label) label.setAttribute("aria-hidden", "true");
  });

  var recordsView = document.querySelector("[data-records-view]");
  if (recordsView) {
    var addRow = recordsView.querySelector("[data-add-record-row]");
    var addToggle = recordsView.querySelector("[data-add-record-toggle]");
    var filterInput = recordsView.querySelector("[data-record-filter]");
    var typeButtons = recordsView.querySelectorAll("[data-type-filter]");
    var records = recordsView.querySelectorAll("[data-record]");
    var noMatches = recordsView.querySelector("[data-no-matches]");
    var countLabel = recordsView.querySelector("[data-record-count]");
    var activeType = "all";

    function applyFilters() {
      var term = (filterInput ? filterInput.value : "").trim().toLowerCase();
      var shown = 0;
      records.forEach(function (row) {
        var matchesType = activeType === "all" || row.dataset.type === activeType;
        var matchesTerm = !term || row.dataset.search.indexOf(term) !== -1;
        var visible = matchesType && matchesTerm;
        row.hidden = !visible;
        if (visible) shown += 1;
      });
      if (noMatches) noMatches.hidden = shown !== 0 || records.length === 0;
      if (countLabel) {
        countLabel.textContent =
          shown === records.length
            ? records.length + " records"
            : shown + " of " + records.length + " records";
      }
    }

    function showAddRow(show) {
      if (!addRow) return;
      addRow.hidden = !show;
      if (show) {
        var first = addRow.querySelector("[data-add-record-first]");
        if (first) first.focus();
      }
    }

    if (addToggle) {
      addToggle.addEventListener("click", function () {
        showAddRow(addRow.hidden);
      });
    }

    typeButtons.forEach(function (button) {
      button.addEventListener("click", function () {
        activeType = button.dataset.typeFilter;
        typeButtons.forEach(function (other) {
          other.setAttribute("aria-pressed", other === button ? "true" : "false");
        });
        applyFilters();
      });
    });

    if (filterInput) filterInput.addEventListener("input", applyFilters);

    // N new, / filter, Esc cancel. Ignored while typing in a field.
    document.addEventListener("keydown", function (event) {
      var target = event.target;
      var typing =
        target &&
        (target.tagName === "INPUT" ||
          target.tagName === "TEXTAREA" ||
          target.tagName === "SELECT" ||
          target.isContentEditable);

      if (event.key === "Escape") {
        if (addRow && !addRow.hidden) {
          showAddRow(false);
          if (addToggle) addToggle.focus();
        } else if (typing && target === filterInput) {
          filterInput.value = "";
          applyFilters();
          filterInput.blur();
        }
        return;
      }

      if (typing || event.metaKey || event.ctrlKey || event.altKey) return;

      if (event.key === "n" || event.key === "N") {
        if (addRow) {
          event.preventDefault();
          showAddRow(true);
        }
      } else if (event.key === "/") {
        if (filterInput) {
          event.preventDefault();
          filterInput.focus();
        }
      }
    });
  }

  // --- Dashboard overview ------------------------------------------------

  var overview = document.querySelector("[data-overview]");
  if (overview) {
    var domainRow = overview.querySelector("[data-add-domain-row]");
    var domainFilter = overview.querySelector("[data-domain-filter]");
    var domains = overview.querySelectorAll("[data-domain]");
    var noDomainMatches = overview.querySelector("[data-no-domain-matches]");

    function showDomainRow(show) {
      if (!domainRow) return;
      domainRow.hidden = !show;
      if (show) {
        var first = domainRow.querySelector("[data-add-domain-first]");
        if (first) first.focus();
      }
    }

    overview.querySelectorAll("[data-add-domain-toggle]").forEach(function (btn) {
      btn.addEventListener("click", function () {
        showDomainRow(domainRow && domainRow.hidden);
      });
    });
    overview.querySelectorAll("[data-add-domain-cancel]").forEach(function (btn) {
      btn.addEventListener("click", function () {
        showDomainRow(false);
      });
    });

    if (domainFilter) {
      domainFilter.addEventListener("input", function () {
        var term = domainFilter.value.trim().toLowerCase();
        var shown = 0;
        domains.forEach(function (row) {
          var match = !term || row.dataset.search.indexOf(term) !== -1;
          row.hidden = !match;
          if (match) shown += 1;
        });
        if (noDomainMatches) {
          noDomainMatches.hidden = shown !== 0 || domains.length === 0;
        }
      });
    }

    // D opens "Add domain", Esc closes it. Ignored while typing.
    document.addEventListener("keydown", function (event) {
      var target = event.target;
      var typing =
        target &&
        (target.tagName === "INPUT" ||
          target.tagName === "TEXTAREA" ||
          target.tagName === "SELECT" ||
          target.isContentEditable);

      if (event.key === "Escape" && domainRow && !domainRow.hidden) {
        showDomainRow(false);
        return;
      }
      if (typing || event.metaKey || event.ctrlKey || event.altKey) return;
      if (event.key === "d" || event.key === "D") {
        if (domainRow) {
          event.preventDefault();
          showDomainRow(true);
        }
      }
    });
  }

  // Forms that destroy something ask first.
  document.querySelectorAll("form[data-confirm]").forEach(function (form) {
    form.addEventListener("submit", function (event) {
      if (!window.confirm(form.dataset.confirm)) {
        event.preventDefault();
      }
    });
  });

  // The show-once token banner can be dismissed; the value is never re-fetched.
  document.querySelectorAll("[data-token-dismiss]").forEach(function (button) {
    button.addEventListener("click", function () {
      var banner = button.closest("[data-token-reveal]");
      if (banner) banner.hidden = true;
    });
  });

  // Account menu in the top bar. Holds sign out, so it exists on every page.
  document.querySelectorAll("[data-account-menu]").forEach(function (button) {
    var panel = document.getElementById(button.getAttribute("aria-controls"));
    if (!panel) return;
    function setOpen(open) {
      panel.hidden = !open;
      button.setAttribute("aria-expanded", open ? "true" : "false");
    }
    button.addEventListener("click", function (event) {
      event.stopPropagation();
      setOpen(panel.hidden);
    });
    document.addEventListener("click", function (event) {
      if (!panel.hidden && !panel.contains(event.target)) setOpen(false);
    });
    document.addEventListener("keydown", function (event) {
      if (event.key === "Escape" && !panel.hidden) {
        setOpen(false);
        button.focus();
      }
    });
  });

  // The dashboard search control focuses the filter available on the current page.
  function focusDashboardSearch() {
    var input = document.querySelector("[data-domain-filter], [data-record-filter]");
    if (input) {
      input.focus();
      return;
    }
    window.location.href = "/dash/domains";
  }

  document.querySelectorAll("[data-zone-search]").forEach(function (button) {
    button.addEventListener("click", focusDashboardSearch);
  });

  document.addEventListener("keydown", function (event) {
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
      event.preventDefault();
      focusDashboardSearch();
    }
  });

  // Domain switcher in the zone top bar.
  document.querySelectorAll("[data-zone-switcher]").forEach(function (button) {
    var menu = document.getElementById(button.getAttribute("aria-controls"));
    if (!menu) return;
    function setOpen(open) {
      menu.hidden = !open;
      button.setAttribute("aria-expanded", open ? "true" : "false");
    }
    button.addEventListener("click", function (event) {
      event.stopPropagation();
      setOpen(menu.hidden);
    });
    document.addEventListener("click", function (event) {
      if (!menu.hidden && !menu.contains(event.target)) setOpen(false);
    });
    document.addEventListener("keydown", function (event) {
      if (event.key === "Escape" && !menu.hidden) setOpen(false);
    });
  });

  // REST / gRPC switcher in the API section.
  document.querySelectorAll("[data-code-tab]").forEach(function (tab) {
    tab.addEventListener("click", function () {
      var group = tab.closest(".code-window");
      if (!group) return;
      group.querySelectorAll("[data-code-tab]").forEach(function (other) {
        other.setAttribute("aria-selected", other === tab ? "true" : "false");
      });
      group.querySelectorAll("[data-code-panel]").forEach(function (panel) {
        panel.hidden = panel.dataset.codePanel !== tab.dataset.codeTab;
      });
    });
  });
})();
