import type { ComponentProps } from 'react';
import { MDXContent } from '@content-collections/mdx/react';

/**
 * Type for the `components` prop accepted by <MDXContent />
 * (avoids guessing and stays in sync with the library)
 */
export type MdxComponents = NonNullable<
  ComponentProps<typeof MDXContent>['components']
>;

/**
 * Default styled element mappings for MDX; you can pass overrides if needed.
 */
export function mdxComponents(
  overrides: Partial<MdxComponents> = {}
): MdxComponents {
  return {
    h1: (p) => <h1 className="mdx-h1" {...p} />,
    h2: (p) => <h2 className="mdx-h2" {...p} />,
    h3: (p) => <h3 className="mdx-h3" {...p} />,
    p: (p) => <p className="mdx-p" {...p} />,
    a: (p) => <a className="mdx-a" {...p} />,
    code: (p) => <code className="mdx-code" {...p} />,
    pre: (p) => <pre className="mdx-pre" {...p} />,
    ul: (p) => <ul className="mdx-ul" {...p} />,
    ol: (p) => <ol className="mdx-ol" {...p} />,
    li: (p) => <li className="mdx-li" {...p} />,
    blockquote: (p) => <blockquote className="mdx-blockquote" {...p} />,
    table: (p) => <table className="mdx-table" {...p} />,
    thead: (p) => <thead className="mdx-thead" {...p} />,
    th: (p) => <th className="mdx-th" {...p} />,
    td: (p) => <td className="mdx-td" {...p} />,
    hr: (p) => <hr className="mdx-hr" {...p} />,
    ...overrides,
  };
}
