import { allPosts } from 'content-collections';
import { notFound } from 'next/navigation';
import MdxWrapper from '@/app/_components/mdx/MdxWrapper';
import DoubleSlitFraunhofer from '@/app/_components/sims/DoubleSlit/DoubleSlitFraunhofer';

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

  // Custom components available in MDX
  const customComponents = {
    DoubleSlitFraunhofer, // <-- expose to MDX
  };

  return (
    <article className="prose prose-invert bg-background/80 mx-4 max-w-3xl rounded-lg p-6 py-10 sm:mx-0 sm:p-8 lg:mx-auto dark:bg-transparent dark:p-0 dark:sm:p-0">
      <h1 className="text-foreground mb-2">{post.title}</h1>
      <p className="text-muted-foreground text-sm">
        {new Date(post.date).toLocaleDateString()}
      </p>
      <MdxWrapper code={post.mdx} customComponents={customComponents} />
    </article>
  );
}
