import Link from 'next/link';
import ThemeToggle from './ThemeToggle';
import { cookies } from 'next/headers';

export default async function Header() {
  const cookieStore = await cookies();
  const themeCookie = cookieStore.get('theme')?.value;
  const initialTheme =
    themeCookie === 'dark' || themeCookie === 'light' ? themeCookie : undefined;
  return (
    <header className="sticky top-6 z-50 mx-auto mt-6 max-w-5xl px-4">
      <div
        className="border-border bg-card/80 supports-[backdrop-filter]:bg-card/60 motion-safe:animate-fade-in relative grid grid-cols-[auto_1fr_auto] items-center gap-2 rounded-2xl border p-2 shadow-lg backdrop-blur motion-reduce:animate-none"
        role="navigation"
        aria-label="Main"
      >
        {/* Left: Home */}
        <Link
          href="/"
          className="group text-foreground hover:bg-accent/20 hover:text-foreground inline-flex items-center gap-2 rounded-xl px-3 py-2 text-sm font-medium transition-colors motion-reduce:transition-none"
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
            className="border-border bg-background/80 text-foreground placeholder:text-muted-foreground focus:ring-ring mx-auto w-full max-w-md rounded-xl border px-4 py-2 text-sm shadow-inner focus:border-transparent focus:ring-2 focus:outline-none"
            aria-label="Search posts"
          />
        </div>

        {/* Right: Theme toggle */}
        <div className="px-2">
          <ThemeToggle className="rounded-full" initialTheme={initialTheme} />
        </div>

        {/* Subtle accent bar */}
        <div
          className="from-primary/60 to-accent/60 pointer-events-none absolute top-1/2 -left-1 h-6 w-1 -translate-y-1/2 rounded-full bg-gradient-to-b"
          aria-hidden="true"
        />
      </div>
    </header>
  );
}
