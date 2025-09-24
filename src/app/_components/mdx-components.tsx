import type { ComponentProps } from "react";
import { MDXContent } from "@content-collections/mdx/react";

/**
 * Type for the `components` prop accepted by <MDXContent />
 * (avoids guessing and stays in sync with the library)
 */
export type MdxComponents = NonNullable<
  ComponentProps<typeof MDXContent>["components"]
>;

/**
 * Default styled element mappings for MDX; you can pass overrides if needed.
 */
export function mdxComponents(
  overrides: Partial<MdxComponents> = {}
): MdxComponents {
  return {
    h1: (p) => (
      <h1
        className="text-3xl font-bold mb-4 text-foreground tracking-tight"
        {...p}
      />
    ),
    h2: (p) => (
      <h2
        className="text-2xl font-semibold mt-8 mb-3 text-foreground"
        {...p}
      />
    ),
    h3: (p) => (
      <h3 className="text-xl font-semibold mt-6 mb-2 text-foreground" {...p} />
    ),
    p: (p) => (
      <p className="my-4 leading-7 text-muted-foreground" {...p} />
    ),
    a: (p) => (
      <a
        className="text-primary underline underline-offset-2 hover:opacity-90"
        {...p}
      />
    ),
    code: (p) => (
      <code
        className="rounded px-1.5 py-0.5 bg-muted text-foreground"
        {...p}
      />
    ),
    pre: (p) => (
      <pre
        className="rounded-xl p-4 overflow-x-auto bg-card text-foreground border border-border"
        {...p}
      />
    ),
    ul: (p) => <ul className="list-disc pl-6 my-4 space-y-1" {...p} />,
    ol: (p) => <ol className="list-decimal pl-6 my-4 space-y-1" {...p} />,
    li: (p) => <li className="text-muted-foreground" {...p} />,
    blockquote: (p) => (
      <blockquote
        className="border-l-4 pl-4 italic bg-card/40 text-muted-foreground border-border"
        {...p}
      />
    ),
    table: (p) => (
      <table
        className="w-full text-sm my-4 border-separate border-spacing-y-1 text-foreground"
        {...p}
      />
    ),
    thead: (p) => <thead className="bg-muted/50" {...p} />,
    th: (p) => (
      <th className="text-left font-semibold px-2 py-1 text-foreground" {...p} />
    ),
    td: (p) => (
      <td className="px-2 py-1 text-muted-foreground" {...p} />
    ),
    hr: (p) => <hr className="my-8 border-border" {...p} />,
    ...overrides,
  };
}
