'use client';

import { useEffect, useState } from 'react';
import { flushSync } from 'react-dom';

type Theme = 'light' | 'dark';

function applyTheme(theme: Theme) {
  const root = document.documentElement;
  root.classList.toggle('dark', theme === 'dark');
  root.style.colorScheme = theme;
}

export default function ThemeToggle({
  className,
  initialTheme,
}: {
  className?: string;
  initialTheme?: Theme;
}) {
  const [theme, setTheme] = useState<Theme>(() => {
    if (typeof document === 'undefined') return initialTheme ?? 'light';
    return document.documentElement.classList.contains('dark')
      ? 'dark'
      : 'light';
  });
  const [rotation, setRotation] = useState<number>(() => {
    if (typeof document === 'undefined') {
      return (initialTheme ?? 'light') === 'dark' ? 180 : 0;
    }
    return document.documentElement.classList.contains('dark') ? 180 : 0;
  });
  const isDark = theme === 'dark';

  // Sync internal state to actual DOM class on mount without changing visuals
  useEffect(() => {
    if (typeof document === 'undefined') return;
    const isDarkNow = document.documentElement.classList.contains('dark');
    setTheme(isDarkNow ? 'dark' : 'light');
    setRotation(isDarkNow ? 180 : 0);
  }, []);

  const toggleTheme = (next: Theme) => {
    if (typeof document === 'undefined') {
      setTheme(next);
      return;
    }

    const reduceMotion = window.matchMedia(
      '(prefers-reduced-motion: reduce)'
    ).matches;

    if (document.startViewTransition && !reduceMotion) {
      const vt = document.startViewTransition(() => {
        // Apply DOM change inside the callback so the browser snapshots properly
        applyTheme(next);
        document.cookie = `theme=${next}; path=/; max-age=31536000; samesite=lax`;
        flushSync(() => {
          setTheme(next);
          setRotation((r) => r + 180);
        });
      });
      document.documentElement.classList.add('vt-active');
      vt.finished.finally(() => {
        document.documentElement.classList.remove('vt-active');
      });
    }
  };

  return (
    <button
      type="button"
      role="switch"
      aria-checked={isDark}
      onClick={() => toggleTheme(isDark ? 'light' : 'dark')}
      onKeyDown={(e) => {
        if (e.key === ' ' || e.key === 'Enter') {
          e.preventDefault();
          toggleTheme(isDark ? 'light' : 'dark');
        }
      }}
      style={{ viewTransitionName: 'theme-toggle' }}
      className={
        'group bg-card text-foreground hover:border-accent focus-visible:ring-ring inline-flex cursor-pointer items-center rounded-full shadow-sm transition-colors focus-visible:ring-2 focus-visible:outline-none motion-reduce:transition-none' +
        (className ? ' ' + className : '')
      }
      aria-label="Toggle dark mode"
      title="Toggle dark mode"
    >
      <span className="sr-only">Toggle dark mode</span>
      {/* Outer container for both layers */}
      <span className="relative inline-flex h-12 w-12">
        {/* Static viewport window - transparent now, just for shadow/depth */}
        <span className="relative z-10 inline-flex h-12 w-12 items-center justify-center rounded-full">
          {/* Circular viewport cutout to see through */}
          <span className="relative z-20 flex h-12 w-12 items-center justify-center overflow-hidden rounded-full">
            {/* Rotating dial with split colored background */}
            <span
              className="theme-dial absolute top-1 left-1/2 h-28 w-28 -translate-x-1/2 -translate-y-[7px] transition-transform duration-700 ease-in-out motion-reduce:transition-none"
              style={{ transform: `rotate(${rotation}deg)` }}
            >
              {/* Split background circle - top half blue (sun), bottom half slate (moon) */}
              <span className="absolute inset-0 overflow-hidden rounded-full">
                {/* Top half - blue gradient for sun */}
                <span
                  className="absolute top-0 left-0 h-1/2 w-full bg-gradient-to-b from-sky-100 to-blue-200"
                  style={{ transformOrigin: 'bottom center' }}
                />
                {/* Bottom half - slate gradient for moon */}
                <span
                  className="absolute bottom-0 left-0 h-1/2 w-full bg-gradient-to-b from-slate-800 to-slate-900"
                  style={{ transformOrigin: 'top center' }}
                />
              </span>

              {/* Sun - at top of dial, centered in viewport */}
              <span className="absolute top-4 left-1/2 z-10 -translate-x-1/2">
                <svg
                  className="h-6 w-6 text-yellow-500 drop-shadow-[0_0_8px_rgba(234,179,8,0.6)] transition-transform duration-700 ease-in-out motion-reduce:transition-none"
                  style={{ transform: `rotate(${-rotation}deg)` }}
                  fill="currentColor"
                  viewBox="0 0 24 24"
                  xmlns="http://www.w3.org/2000/svg"
                >
                  <circle cx="12" cy="12" r="4" />
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth="2"
                    stroke="currentColor"
                    fill="none"
                    d="M12 2v2m0 16v2M4.93 4.93l1.41 1.41m11.32 11.32l1.41 1.41M2 12h2m16 0h2M4.93 19.07l1.41-1.41m11.32-11.32l1.41-1.41"
                  />
                </svg>
              </span>
              {/* Moon - at bottom of dial */}
              <span className="absolute bottom-4 left-1/2 z-10 -translate-x-1/2">
                <svg
                  className="h-6 w-6 text-slate-200 drop-shadow-[0_0_8px_rgba(226,232,240,0.4)] transition-transform duration-700 ease-in-out motion-reduce:transition-none"
                  style={{ transform: `rotate(${-rotation}deg)` }}
                  fill="currentColor"
                  viewBox="0 0 24 24"
                  xmlns="http://www.w3.org/2000/svg"
                >
                  <path d="M21.64 13a1 1 0 0 0-1.05-.14 8.05 8.05 0 0 1-3.37.73 8.15 8.15 0 0 1-8.14-8.1 8.59 8.59 0 0 1 .25-2A1 1 0 0 0 8 2.36a10.14 10.14 0 1 0 14 11.69 1 1 0 0 0-.36-1.05z" />
                </svg>
              </span>
            </span>
          </span>
        </span>
      </span>
    </button>
  );
}
