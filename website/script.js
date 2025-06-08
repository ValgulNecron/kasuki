document.addEventListener('DOMContentLoaded', () => {

    // --- Mobile Menu Toggle ---
    const menuToggle = document.querySelector('.menu-toggle');
    const navLinks = document.querySelector('.nav-links');
    if (menuToggle && navLinks) {
        menuToggle.addEventListener('click', () => {
            menuToggle.classList.toggle('active');
            navLinks.classList.toggle('active');
        });

        // Close mobile menu when clicking a link
        navLinks.querySelectorAll('a').forEach(item => {
            item.addEventListener('click', () => {
                if (navLinks.classList.contains('active')) {
                    menuToggle.classList.remove('active');
                    navLinks.classList.remove('active');
                }
            });
        });
    }

    // --- Command Tabs Functionality ---
    const commandTabs = document.querySelectorAll('.command-tab');
    const commandGroups = document.querySelectorAll('.command-group');
    if (commandTabs.length > 0 && commandGroups.length > 0) {
        commandTabs.forEach(tab => {
            tab.addEventListener('click', () => {
                // Deactivate all tabs and groups
                commandTabs.forEach(t => t.classList.remove('active'));
                commandGroups.forEach(group => group.classList.remove('active'));

                // Activate clicked tab and corresponding group
                tab.classList.add('active');
                const targetGroup = document.querySelector(`.command-group[data-tab="${tab.dataset.tab}"]`);
                if (targetGroup) {
                    targetGroup.classList.add('active');
                }
            });
        });
    }

    // --- Smooth Scrolling for Anchor Links ---
    document.querySelectorAll('a[href^="#"]').forEach(anchor => {
        anchor.addEventListener('click', function (e) {
            const href = this.getAttribute('href');
            // Ensure it's a valid anchor link on the page
            if (href.length > 1 && document.querySelector(href)) {
                e.preventDefault();
                const targetElement = document.querySelector(href);
                const headerOffset = 80; // Height of the fixed header
                const elementPosition = targetElement.getBoundingClientRect().top;
                const offsetPosition = elementPosition + window.pageYOffset - headerOffset;

                window.scrollTo({
                    top: offsetPosition,
                    behavior: 'smooth'
                });
            }
        });
    });

    // --- Animate elements on scroll ---
    const scrollElements = document.querySelectorAll('.feature-card, .screenshot, .step');
    if (scrollElements.length > 0) {
        const elementInView = (el, dividend = 1) => {
            const elementTop = el.getBoundingClientRect().top;
            return (
                elementTop <= (window.innerHeight || document.documentElement.clientHeight) / dividend
            );
        };

        const displayScrollElement = (element) => {
            element.classList.add('scrolled');
        };

        const handleScrollAnimation = () => {
            scrollElements.forEach((el) => {
                if (elementInView(el, 1.25)) {
                    displayScrollElement(el);
                }
            });
        };
        
        // Add initial styles for animation
        scrollElements.forEach(element => {
            element.style.opacity = '0';
            element.style.transform = 'translateY(20px)';
            element.style.transition = 'opacity 0.6s ease-out, transform 0.6s ease-out';
        });

        // Add 'scrolled' class to apply final styles
        const style = document.createElement('style');
        style.innerHTML = `
            .scrolled {
                opacity: 1 !important;
                transform: translateY(0) !important;
            }
        `;
        document.head.appendChild(style);

        window.addEventListener('scroll', handleScrollAnimation);
        // Initial check on load
        handleScrollAnimation();
    }


    // --- Screenshot Image Modal Functionality ---
    const screenshotImages = document.querySelectorAll('.screenshot img');
    const imageModal = document.querySelector('.image-modal');
    const modalImage = imageModal ? imageModal.querySelector('img') : null;

    if (screenshotImages.length > 0 && imageModal && modalImage) {
        screenshotImages.forEach(screenshot => {
            screenshot.style.cursor = 'pointer';
            screenshot.addEventListener('click', (e) => {
                modalImage.src = e.target.src;
                imageModal.classList.add('active');
            });
        });

        // Close modal when clicking on it
        imageModal.addEventListener('click', () => {
            imageModal.classList.remove('active');
        });
    }

    // --- Theme Toggler Functionality ---
    const themeToggle = document.querySelector('.theme-toggle');
    if (themeToggle) {
        // Function to set the theme
        const setTheme = (isDark) => {
            if (isDark) {
                document.documentElement.setAttribute('data-theme', 'dark');
                themeToggle.innerHTML = '<i class="fas fa-sun"></i>';
                localStorage.setItem('kasuki-theme', 'dark');
            } else {
                document.documentElement.removeAttribute('data-theme');
                themeToggle.innerHTML = '<i class="fas fa-moon"></i>';
                localStorage.setItem('kasuki-theme', 'light');
            }
        };

        // Check for saved theme in localStorage
        let isDarkMode = localStorage.getItem('kasuki-theme') === 'dark';

        // Set initial theme
        setTheme(isDarkMode);

        // Add click event listener
        themeToggle.addEventListener('click', () => {
            isDarkMode = !isDarkMode;
            setTheme(isDarkMode);
        });
    }
    
    // --- Auto-update Copyright Year ---
    const copyrightYear = document.querySelector('.footer-bottom .year');
    if (copyrightYear) {
        copyrightYear.textContent = new Date().getFullYear();
    }

});
