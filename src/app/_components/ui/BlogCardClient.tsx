'use client';

import Link from 'next/link';
import Image from 'next/image';
import { useEffect, useState } from 'react';

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

function formatDate(dateStr: string): string {
  const d = new Date(dateStr);
  if (Number.isNaN(d.getTime())) return dateStr;
  return d.toISOString().slice(0, 10);
}

export default function BlogCardClient({ post }: { post: BlogPost }) {
  const [loaded, setLoaded] = useState(false);
  useEffect(() => {
    if (!post.image) setLoaded(true);
  }, [post.image]);

  return (
    <Link
      href={`/${post._meta.path}`}
      className="group hover:bg-secondary/20 hover:border-secondary-dark dark:hover:border-secondary overflow-hidden rounded-xl border border-zinc-800 transition"
      style={{
        opacity: loaded ? 1 : 0,
        transition: 'opacity 700ms ease-in-out',
      }}
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
            onLoad={() => setLoaded(true)}
            onError={() => setLoaded(true)}
          />
        ) : (
          // If no image, show card immediately
          <span className="sr-only">No image</span>
        )}
      </div>

      {/* Meta */}
      <div className="p-4">
        <h2 className="text-foreground text-lg font-semibold">{post.title}</h2>
        <div className="text-accent-foreground dark:text-accent text-sm">
          {formatDate(post.date)}
          {post.tags?.length ? ` · ${post.tags.join(' · ')}` : ''}
        </div>
        <p className="mt-2 line-clamp-3 text-zinc-600 dark:text-zinc-300">
          {post.description}
        </p>
      </div>
    </Link>
  );
}
