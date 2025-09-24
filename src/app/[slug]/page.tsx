import { allPosts } from "content-collections";
import { notFound } from "next/navigation";
import { MDXContent } from "@content-collections/mdx/react";
import { mdxComponents } from "@/app/_components/mdx-components";
import DoubleSlitFraunhofer from "@/app/_components/sims/DoubleSlitFraunhofer";

export const dynamicParams = false;

export async function generateStaticParams() {
  return allPosts.filter((p) => !p.draft).map((p) => ({ slug: p._meta.path }));
}


export default function PostPage({ params }: { params: { slug: string } }) {
  const post = allPosts.find(p => p._meta.path === params.slug);
  if (!post) return notFound();

  // Merge your base MDX element styles + custom components you want available in MDX
  const mergedComponents = {
    ...mdxComponents(),
    DoubleSlitFraunhofer, // <-- expose to MDX
  };

  return (
    <article className="mx-auto max-w-3xl py-10 prose prose-invert">
      <h1 className="mb-2 text-foreground">{post.title}</h1>
  <p className="text-sm text-muted-foreground">
    {new Date(post.date).toLocaleDateString()}
  </p>
  <MDXContent code={post.mdx} components={mergedComponents} />
  </article>
  );
}
