import { defineCollection, defineConfig } from '@content-collections/core';
import { compileMDX } from '@content-collections/mdx';
import remarkGfm from 'remark-gfm';
import remarkMath from 'remark-math';
import rehypeKatex from 'rehype-katex';
import rehypePrettyCode from 'rehype-pretty-code';
import { z } from 'zod';

const posts = defineCollection({
  name: 'posts',
  directory: 'content/posts',
  include: '**/*.mdx',
  schema: z.object({
    title: z.string(),
    description: z.string(),
    date: z.string(),
    tags: z.array(z.string()).optional(),
    draft: z.boolean().default(false),
    image: z.string().optional(), // <-- just a path like "/images/foo.jpg"
    imageAlt: z.string().optional(), // <-- optional alt text (recommended)
  }),
  transform: async (document, context) => {
    // ✅ compileMDX returns a string
    const mdx = await compileMDX(context, document, {
      remarkPlugins: [remarkGfm, remarkMath],
      rehypePlugins: [
        [rehypeKatex, { trust: true, strict: false }],
        [
          rehypePrettyCode,
          {
            theme: {
              light: 'github-light',
              dark: 'github-dark',
            },
            keepBackground: false,
            defaultLang: 'plaintext',
          },
        ],
      ],
    });

    return { ...document, mdx }; // <- store the string on the doc
  },
});

export default defineConfig({ collections: [posts] });
