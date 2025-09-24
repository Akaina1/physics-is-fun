import Link from "next/link";
import Image from "next/image";
import { allPosts } from "content-collections";

export default function Page() {
  const posts = allPosts
    .filter((p) => !p.draft)
    .sort((a, b) => +new Date(b.date) - +new Date(a.date));

  return (
    <div className="mx-auto max-w-5xl py-10">
      <h1 className="text-3xl font-bold mb-6">Research Log</h1>
      <div className="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
        {posts.map((p) => (
          <Link
            key={p._meta.filePath}
            href={`/${p._meta.path}`}
            className="group rounded-xl border border-zinc-800 overflow-hidden hover:border-zinc-700 transition"
          >
            {/* Image */}
            <div className="relative aspect-[16/9] bg-zinc-900">
              {p.image ? (
                <Image
                  src={p.image}
                  alt={p.imageAlt ?? p.title}
                  fill
                  sizes="(min-width: 1024px) 33vw, (min-width: 640px) 50vw, 100vw"
                  className="object-cover"
                  priority={false}
                />
              ) : null}
            </div>

            {/* Meta */}
            <div className="p-4">
              <h2 className="text-lg font-semibold group-hover:underline">
                {p.title}
              </h2>
              <div className="text-sm text-zinc-400">
                {new Date(p.date).toLocaleDateString()}
                {p.tags?.length ? ` · ${p.tags.join(" · ")}` : ""}
              </div>
              <p className="mt-2 text-zinc-300 line-clamp-3">{p.description}</p>
            </div>
          </Link>
        ))}
      </div>
    </div>
  );
}
