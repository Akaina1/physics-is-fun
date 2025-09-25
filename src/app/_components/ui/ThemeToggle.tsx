'use client';

import { useEffect, useState } from 'react';

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
  const isDark = theme === 'dark';

  // Sync internal state to actual DOM class on mount without changing visuals (which are CSS-driven)
  useEffect(() => {
    if (typeof document === 'undefined') return;
    setTheme(
      document.documentElement.classList.contains('dark') ? 'dark' : 'light'
    );
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
      document.startViewTransition(() => {
        // Apply DOM change inside the callback so the browser snapshots properly
        applyTheme(next);
        document.cookie = `theme=${next}; path=/; max-age=31536000; samesite=lax`;
        setTheme(next);
      });
    } else {
      // Fallback with smooth CSS transition support
      const root = document.documentElement;
      root.classList.add('changing-theme');
      applyTheme(next);
      document.cookie = `theme=${next}; path=/; max-age=31536000; samesite=lax`;
      setTheme(next);
      window.setTimeout(() => root.classList.remove('changing-theme'), 450);
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
      className={
        'group border-border bg-card text-foreground hover:border-accent focus-visible:ring-ring inline-flex cursor-pointer items-center rounded-full border p-1 shadow-sm transition-colors focus-visible:ring-2 focus-visible:outline-none motion-reduce:transition-none' +
        (className ? ' ' + className : '')
      }
      aria-label="Toggle dark mode"
      title="Toggle dark mode"
    >
      <span className="sr-only">Toggle dark mode</span>
      <span
        className={
          'border-border bg-muted dark:bg-accent/30 relative inline-flex h-6 w-11 items-center rounded-full border shadow-inner transition-colors motion-reduce:transition-none'
        }
      >
        {/* Icons for context */}
        <span className="text-muted-foreground pointer-events-none absolute left-1 text-[10px] leading-none">
          ☀️
        </span>
        <span className="text-muted-foreground pointer-events-none absolute right-1 text-[10px] leading-none">
          🌙
        </span>
        {/* Thumb */}
        <span
          className={
            'bg-card ring-border inline-block h-5 w-5 translate-x-0.5 transform rounded-full shadow-sm ring-1 transition-transform motion-reduce:transition-none dark:translate-x-5'
          }
        />
      </span>
    </button>
  );
}
