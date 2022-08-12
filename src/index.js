(function () {
    var DARK_THEME_CLASSNAME = "dark";

    var themeToggle = document.getElementById("theme-checkbox");

    themeToggle.addEventListener('change', function () {
        document.documentElement.classList.toggle(DARK_THEME_CLASSNAME);
    });
})();
