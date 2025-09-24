import Link from "next/link";

export type FooterLink = {
  key: string;
  displayText: string;
  href: string;
};

export default function Footer({ links }: { links: FooterLink[] }) {
  return (
    <footer className="fixed inset-x-0 bottom-0 z-40">
      <div className="group relative mx-auto w-fit">
        {/* Revealed panel */}
        <div className="absolute bottom-9 left-1/2 -translate-x-1/2">
          <nav
            className="pointer-events-auto mx-auto flex min-w-[280px] max-w-3xl items-center justify-evenly gap-2 rounded-2xl border border-border bg-card/80 px-4 py-2 text-sm text-muted-foreground shadow-lg backdrop-blur supports-[backdrop-filter]:bg-card/60 opacity-0 scale-95 translate-y-2 transition-all motion-reduce:transition-none group-hover:opacity-100 group-hover:scale-100 group-hover:translate-y-0 group-focus-within:opacity-100 group-focus-within:scale-100 group-focus-within:translate-y-0"
            aria-label="Footer links"
          >
            {links.map((link) => (
              <Link
                key={link.key}
                href={link.href}
                className="cursor-pointer inline-flex items-center gap-2 rounded-md bg-card px-3 py-1.5 text-base text-foreground transition-colors motion-reduce:transition-none hover:bg-accent/20 dark:hover:bg-accent/25 hover:border-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring hover:shadow-md"
              >
                {link.displayText}
              </Link>
            ))}
          </nav>
        </div>

        {/* Handle / tab */}
        <button
          type="button"
          className="pointer-events-auto mx-auto inline-flex items-center gap-2 rounded-t-xl rounded-b-none border border-border bg-card/80 px-3 py-1 text-base text-foreground shadow-md backdrop-blur supports-[backdrop-filter]:bg-card/60 transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          aria-label="Show footer links"
        >
          Links
          <span aria-hidden>▴</span>
        </button>
      </div>
    </footer>
  );
}
