(function(){
  const saved=localStorage.getItem('mps-docs-lang')||'zh';
  function setLang(lang){
    const selected=lang==='en'?'en':'zh';
    document.documentElement.lang=selected==='zh'?'zh-CN':'en';
    document.documentElement.dataset.lang=selected;
    document.querySelectorAll('[data-lang]').forEach(x=>{
      x.hidden=x.dataset.lang!==selected;
    });
    document.querySelectorAll('#language-select').forEach(selector=>{
      selector.value=selected;
    });
    // Update nav active state — highlight the pill whose text matches current page
    const path = window.location.pathname;
    document.querySelectorAll('#top-nav .nav-pill').forEach(pill => {
      const href = pill.getAttribute('href');
      const isActive = href === path.substring(path.lastIndexOf('/') + 1) ||
                       (href === 'index.html' && (path.endsWith('/') || path.endsWith('/docs/')));
      pill.classList.toggle('active', isActive);
    });
    localStorage.setItem('mps-docs-lang',selected);
  }
  document.addEventListener('DOMContentLoaded',()=>{
    document.querySelectorAll('#language-select').forEach(selector=>{
      selector.addEventListener('change',()=>setLang(selector.value));
    });
    setLang(saved);
    if(window.hljs) document.querySelectorAll('pre code').forEach(x=>hljs.highlightElement(x));
  });
})();