import Link from 'next/link';

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
            className="border-border bg-card/80 text-muted-foreground supports-[backdrop-filter]:bg-card/60 pointer-events-auto mx-auto flex max-w-3xl min-w-[280px] translate-y-2 scale-95 items-center justify-evenly gap-2 rounded-2xl border px-4 py-2 text-sm opacity-0 shadow-lg backdrop-blur transition-all group-focus-within:translate-y-0 group-focus-within:scale-100 group-focus-within:opacity-100 group-hover:translate-y-0 group-hover:scale-100 group-hover:opacity-100 motion-reduce:transition-none"
            aria-label="Footer links"
          >
            {links.map((link) => (
              <Link
                key={link.key}
                href={link.href}
                className="bg-card text-foreground hover:bg-accent/20 dark:hover:bg-accent/25 hover:border-accent focus-visible:ring-ring inline-flex cursor-pointer items-center gap-2 rounded-md px-3 py-1.5 text-base transition-colors hover:shadow-md focus-visible:ring-2 focus-visible:outline-none motion-reduce:transition-none"
              >
                {link.displayText}
              </Link>
            ))}
          </nav>
        </div>

        {/* Handle / tab */}
        <button
          type="button"
          className="border-border bg-card/80 text-foreground supports-[backdrop-filter]:bg-card/60 focus-visible:ring-ring pointer-events-auto mx-auto inline-flex items-center gap-2 rounded-t-xl rounded-b-none border px-3 py-1 text-base shadow-md backdrop-blur transition-colors focus-visible:ring-2 focus-visible:outline-none motion-reduce:transition-none"
          aria-label="Show footer links"
        >
          Links
          <span aria-hidden>▴</span>
        </button>
      </div>
    </footer>
  );
}
