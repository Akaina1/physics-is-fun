import Link from "next/link";
import ThemeToggle from "./ThemeToggle";
import { cookies } from "next/headers";

export default async function Header() {
  const cookieStore = await cookies();
  const themeCookie = cookieStore.get("theme")?.value;
  const initialTheme = themeCookie === "dark" || themeCookie === "light" ? themeCookie : undefined;
  return (
    <header className="mx-auto mt-6 max-w-5xl px-4 sticky top-6 z-50">
      <div
        className="relative grid grid-cols-[auto_1fr_auto] items-center gap-2 rounded-2xl border border-border bg-card/80 p-2 shadow-lg backdrop-blur supports-[backdrop-filter]:bg-card/60 motion-safe:animate-fade-in motion-reduce:animate-none"
        role="navigation"
        aria-label="Main"
      >
        {/* Left: Home */}
        <Link
          href="/"
          className="group inline-flex items-center gap-2 rounded-xl px-3 py-2 text-sm font-medium text-foreground transition-colors hover:bg-accent/20 hover:text-foreground motion-reduce:transition-none"
          aria-label="Go to home"
        >
          Home
        </Link>

        {/* Center: Search */}
        <div className="px-2">
          <label htmlFor="site-search" className="sr-only">
            Search posts
          </label>
          <input
            id="site-search"
            type="search"
            placeholder="Search physics, astrophysics, quantum..."
            className="mx-auto w-full max-w-md rounded-xl border border-border bg-background/80 px-4 py-2 text-sm text-foreground placeholder:text-muted-foreground shadow-inner focus:border-transparent focus:outline-none focus:ring-2 focus:ring-ring"
            aria-label="Search posts"
          />
        </div>

        {/* Right: Theme toggle */}
        <div className="px-2">
          <ThemeToggle className="rounded-full" initialTheme={initialTheme} />
        </div>

        {/* Subtle accent bar */}
        <div
          className="pointer-events-none absolute -left-1 top-1/2 h-6 w-1 -translate-y-1/2 rounded-full from-primary/60 to-accent/60 bg-gradient-to-b"
          aria-hidden="true"
        />
      </div>
    </header>
  );
}
