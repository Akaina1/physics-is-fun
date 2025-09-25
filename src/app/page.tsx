import { allPosts } from 'content-collections';
import BlogCard from './_components/ui/BlogCard';

export default function Page() {
  const posts = allPosts
    .filter((p) => !p.draft)
    .sort((a, b) => +new Date(b.date) - +new Date(a.date));

  return (
    <div className="mx-4 max-w-5xl py-10 sm:mx-0 lg:mx-auto">
      <h1 className="mb-6 text-3xl font-bold">Research Log</h1>
      <div className="grid gap-6 sm:grid-cols-1 lg:grid-cols-3">
        {posts.map((post) => (
          <BlogCard key={post._meta.filePath} post={post} />
        ))}
      </div>
    </div>
  );
}
