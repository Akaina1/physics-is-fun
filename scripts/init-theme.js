// Do not reference DOM during import; export the script as a string.
const themeInitScript = `(function(){try{var d=document.documentElement;var m=window.matchMedia('(prefers-color-scheme: dark)').matches?'dark':'light';var match=document.cookie.match(/(?:^|; )theme=([^;]*)/);var c=match?decodeURIComponent(match[1]):null;var theme=(c==='dark'||c==='light')?c:m;d.classList.toggle('dark',theme==='dark');d.style.colorScheme=theme;}catch(e){}})();`;

export default themeInitScript;
