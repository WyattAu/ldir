// Version selector logic
(function() {
    const versions = ['v0.1.0 (current)', 'v0.0.1'];
    const select = document.getElementById('version-select');
    if (select) {
        versions.forEach(v => {
            const opt = document.createElement('option');
            opt.value = v;
            opt.textContent = v;
            if (v.includes('current')) opt.selected = true;
            select.appendChild(opt);
        });
        select.addEventListener('change', function() {
            if (!this.value.includes('current')) {
                window.location.href = 'https://github.com/WyattAu/ldir/tree/' + this.value;
            }
        });
    }
    
    // Active sidebar link
    const current = window.location.pathname.split('/').pop() || 'index.html';
    document.querySelectorAll('.sidebar nav a').forEach(a => {
        if (a.getAttribute('href') === current) a.classList.add('active');
    });
})();
