import { allPosts } from "content-collections";
import { notFound } from "next/navigation";
import { MDXContent } from "@content-collections/mdx/react";
import { mdxComponents } from "@/app/_components/mdx-components";

export const dynamicParams = false;

export async function generateStaticParams() {
  return allPosts.filter((p) => !p.draft).map((p) => ({ slug: p._meta.path }));
}

export default async function PostPage({
  params,
}: {
  params: Promise<{ slug: string }>;
}) {
  const { slug } = await params;
  const post = allPosts.find((p) => p._meta.path === slug);
  if (!post) return notFound();

  return (
    <article className="mx-auto max-w-3xl py-10 prose prose-invert">
      <h1 className="mb-2 text-foreground">{post.title}</h1>
  <p className="text-sm text-muted-foreground">
    {new Date(post.date).toLocaleDateString()}
  </p>
  <MDXContent code={post.mdx} components={mdxComponents()} />
  </article>
  );
}
