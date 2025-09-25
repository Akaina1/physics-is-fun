"use client";

import { useEffect, useState } from "react";

type Theme = "light" | "dark";

const STORAGE_KEY = "theme";

function getInitialTheme(): Theme {
  if (typeof window === "undefined") return "light";
  const stored = window.localStorage.getItem(STORAGE_KEY);
  if (stored === "light" || stored === "dark") return stored;
  // fallback: default to light; change to OS preference if you prefer:
  // return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  return "light";
}

function applyTheme(theme: Theme) {
  const root = document.documentElement;
  root.classList.toggle("dark", theme === "dark");
}

export default function ThemeToggle({ className }: { className?: string }) {
  const [theme, setTheme] = useState<Theme>(getInitialTheme);
  const isDark = theme === "dark";
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    const root = document.documentElement;
    const enableSmooth = mounted;

    if (enableSmooth) {
      root.classList.add("changing-theme");
    }

    applyTheme(theme);
    window.localStorage.setItem(STORAGE_KEY, theme);

    if (enableSmooth) {
      const timeoutId = window.setTimeout(() => {
        root.classList.remove("changing-theme");
      }, 400);
      return () => {
        window.clearTimeout(timeoutId);
      };
    }
  }, [theme, mounted]);

  useEffect(() => {
    setMounted(true);
  }, []);

  // Avoid hydration mismatch: render a neutral shell until mounted
  if (!mounted) {
    return (
      <span
        aria-hidden
        className={
          "inline-flex h-6 w-11 items-center rounded-full border border-border bg-muted/60 p-1 shadow-inner " +
          (className ? " " + className : "")
        }
      />
    );
  }

  return (
    <button
      type="button"
      role="switch"
      aria-checked={isDark}
      onClick={() => setTheme(isDark ? "light" : "dark")}
      onKeyDown={(e) => {
        if (e.key === " " || e.key === "Enter") {
          e.preventDefault();
          setTheme((prev) => (prev === "dark" ? "light" : "dark"));
        }
      }}
      className={
        "cursor-pointer group inline-flex items-center rounded-full border border-border bg-card p-1 text-foreground shadow-sm transition-colors motion-reduce:transition-none hover:border-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring" +
        (className ? " " + className : "")
      }
      aria-label="Toggle dark mode"
      title="Toggle dark mode"
    >
      <span className="sr-only">Toggle dark mode</span>
      <span
        className={
          "relative inline-flex h-6 w-11 items-center rounded-full border border-border shadow-inner transition-colors motion-reduce:transition-none " +
          (isDark ? "bg-accent/30" : "bg-muted")
        }
      >
        {/* Icons for context */}
        <span className="pointer-events-none absolute left-1 text-[10px] leading-none text-muted-foreground">☀️</span>
        <span className="pointer-events-none absolute right-1 text-[10px] leading-none text-muted-foreground">🌙</span>
        {/* Thumb */}
        <span
          className={
            "inline-block h-5 w-5 transform rounded-full bg-card ring-1 ring-border shadow-sm transition-transform motion-reduce:transition-none " +
            (isDark ? "translate-x-5" : "translate-x-0.5")
          }
        />
      </span>
    </button>
  );
}
