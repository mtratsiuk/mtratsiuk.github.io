(function () {
    var DARK_THEME_CLASSNAME = "dark";
    var THEME_STORAGE_KEY = "b3-theme"
    var THEMES = {
        JEDI: "jedi",
        SITH: "sith",
    }

    var state = tryExec(function () {
        return window.localStorage.getItem(THEME_STORAGE_KEY)
    }, THEMES.JEDI)

    if (state === THEMES.SITH && !document.documentElement.classList.contains(DARK_THEME_CLASSNAME)) {
        document.documentElement.classList.add(DARK_THEME_CLASSNAME);
    }

    document.addEventListener('DOMContentLoaded', function () {
        var themeToggle = document.getElementById("theme-toggle");

        if (state === THEMES.SITH && themeToggle.textContent !== THEMES.SITH) {
            themeToggle.textContent = THEMES.SITH
        }

        themeToggle.addEventListener("click", function () {
            if (state === THEMES.JEDI) {
                state = THEMES.SITH
                document.documentElement.classList.add(DARK_THEME_CLASSNAME);
            } else {
                state = THEMES.JEDI
                document.documentElement.classList.remove(DARK_THEME_CLASSNAME);
            }

            themeToggle.textContent = state
            tryExec(function () { window.localStorage.setItem(THEME_STORAGE_KEY, state) })
        });
    })

    function tryExec(fn, fallback) {
        try {
            return fn() || fallback
        } catch (err) {
            console.warn(err)
            return fallback
        }
    }
})();
