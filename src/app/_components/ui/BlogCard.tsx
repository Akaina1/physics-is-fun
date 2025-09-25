import BlogCardClient from './BlogCardClient';

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
  return <BlogCardClient post={post} />;
}
