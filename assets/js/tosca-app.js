function burgerMenu() {
    const burgerMenuId = document.getElementById("burger-menu");
    const navMenu = document.querySelector(".nav-menu");
    const body = document.body;

    if (burgerMenuId && navMenu) {
        burgerMenuId.addEventListener("click", () => {
            burgerMenuId.classList.toggle("active");
            navMenu.classList.toggle("active");
            body.classList.toggle("active");
        });
    }

    const navLinks = document.querySelectorAll(".nav-link");
    navLinks.forEach((link) => {
        link.addEventListener("click", () => {
            if (burgerMenuId) burgerMenuId.classList.remove("active");
            if (navMenu) navMenu.classList.remove("active");
            if (body) body.classList.remove("active");
        });
    });
}

document.addEventListener("DOMContentLoaded", () => {
    burgerMenu();
    console.log("Tosca app successfully initialized!");
});

