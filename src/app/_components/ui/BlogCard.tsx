import Link from "next/link";
import Image from "next/image";

interface BlogPost {
  _meta: {
    filePath: string;
    path: string;
  };
  title: string;
  date: string;
  description?: string;
  image?: string;
  imageAlt?: string;
  tags?: string[];
  draft?: boolean;
}

interface BlogCardProps {
  post: BlogPost;
}

export default function BlogCard({ post }: BlogCardProps) {
  return (
    <Link
      href={`/${post._meta.path}`}
      className="group rounded-xl border border-zinc-800 overflow-hidden hover:bg-secondary/20 hover:border-secondary-dark dark:hover:border-secondary transition"
    >
      {/* Image */}
      <div className="relative aspect-[16/9] bg-zinc-900">
        {post.image ? (
          <Image
            src={post.image}
            alt={post.imageAlt ?? post.title}
            fill
            sizes="(min-width: 1024px) 33vw, (min-width: 640px) 50vw, 100vw"
            className="object-cover"
            priority={false}
          />
        ) : null}
      </div>

      {/* Meta */}
      <div className="p-4">
        <h2 className="text-lg font-semibold text-foreground">
          {post.title}
        </h2>
        <div className="text-sm text-accent-foreground dark:text-accent">
          {new Date(post.date).toLocaleDateString()}
          {post.tags?.length ? ` · ${post.tags.join(" · ")}` : ""}
        </div>
        <p className="mt-2 text-zinc-600 dark:text-zinc-300 line-clamp-3">
          {post.description}
        </p>
      </div>
    </Link>
  );
}
