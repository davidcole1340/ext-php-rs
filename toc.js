// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="introduction.html">Introduction</a></li><li class="chapter-item expanded affix "><li class="part-title">Getting Started</li><li class="chapter-item expanded "><a href="getting-started/installation.html"><strong aria-hidden="true">1.</strong> Installation</a></li><li class="chapter-item expanded "><a href="getting-started/hello_world.html"><strong aria-hidden="true">2.</strong> Hello World</a></li><li class="chapter-item expanded "><a href="getting-started/cargo-php.html"><strong aria-hidden="true">3.</strong> cargo php</a></li><li class="chapter-item expanded affix "><li class="part-title">Reference Guide</li><li class="chapter-item expanded "><a href="types/index.html"><strong aria-hidden="true">4.</strong> Types</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="types/numbers.html"><strong aria-hidden="true">4.1.</strong> Primitive Numbers</a></li><li class="chapter-item expanded "><a href="types/string.html"><strong aria-hidden="true">4.2.</strong> String</a></li><li class="chapter-item expanded "><a href="types/str.html"><strong aria-hidden="true">4.3.</strong> &amp;str</a></li><li class="chapter-item expanded "><a href="types/bool.html"><strong aria-hidden="true">4.4.</strong> bool</a></li><li class="chapter-item expanded "><a href="types/vec.html"><strong aria-hidden="true">4.5.</strong> Vec</a></li><li class="chapter-item expanded "><a href="types/hashmap.html"><strong aria-hidden="true">4.6.</strong> HashMap</a></li><li class="chapter-item expanded "><a href="types/binary.html"><strong aria-hidden="true">4.7.</strong> Binary</a></li><li class="chapter-item expanded "><a href="types/binary_slice.html"><strong aria-hidden="true">4.8.</strong> BinarySlice</a></li><li class="chapter-item expanded "><a href="types/option.html"><strong aria-hidden="true">4.9.</strong> Option</a></li><li class="chapter-item expanded "><a href="types/object.html"><strong aria-hidden="true">4.10.</strong> Object</a></li><li class="chapter-item expanded "><a href="types/class_object.html"><strong aria-hidden="true">4.11.</strong> Class Object</a></li><li class="chapter-item expanded "><a href="types/closure.html"><strong aria-hidden="true">4.12.</strong> Closure</a></li><li class="chapter-item expanded "><a href="types/functions.html"><strong aria-hidden="true">4.13.</strong> Functions &amp; methods</a></li></ol></li><li class="chapter-item expanded "><a href="macros/index.html"><strong aria-hidden="true">5.</strong> Macros</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="macros/module.html"><strong aria-hidden="true">5.1.</strong> Module</a></li><li class="chapter-item expanded "><a href="macros/function.html"><strong aria-hidden="true">5.2.</strong> Function</a></li><li class="chapter-item expanded "><a href="macros/classes.html"><strong aria-hidden="true">5.3.</strong> Classes</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="macros/impl.html"><strong aria-hidden="true">5.3.1.</strong> impls</a></li></ol></li><li class="chapter-item expanded "><a href="macros/constant.html"><strong aria-hidden="true">5.4.</strong> Constants</a></li><li class="chapter-item expanded "><a href="macros/extern.html"><strong aria-hidden="true">5.5.</strong> PHP Functions</a></li><li class="chapter-item expanded "><a href="macros/zval_convert.html"><strong aria-hidden="true">5.6.</strong> ZvalConvert</a></li><li class="chapter-item expanded "><a href="macros/php.html"><strong aria-hidden="true">5.7.</strong> Attributes</a></li></ol></li><li class="chapter-item expanded "><a href="exceptions.html"><strong aria-hidden="true">6.</strong> Exceptions</a></li><li class="chapter-item expanded "><a href="ini-settings.html"><strong aria-hidden="true">7.</strong> INI Settings</a></li><li class="chapter-item expanded affix "><li class="part-title">Advanced Topics</li><li class="chapter-item expanded "><a href="advanced/async_impl.html"><strong aria-hidden="true">8.</strong> Async</a></li><li class="chapter-item expanded "><a href="advanced/allowed_bindings.html"><strong aria-hidden="true">9.</strong> Allowed Bindings</a></li><li class="chapter-item expanded affix "><li class="part-title">Migration Guides</li><li class="chapter-item expanded affix "><li class="spacer"></li><li class="chapter-item expanded affix "><a href="migration-guides/v0.14.html">v0.14</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0].split("?")[0];
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
