"use client";

import { memo } from "react";
import { MDXContent } from "@content-collections/mdx/react";
import { mdxComponents, type MdxComponents } from "./MdxComponents";

interface MdxWrapperProps {
  code: string;
  customComponents?: Record<string, React.ComponentType>;
}

// Memoized wrapper to prevent MDX re-rendering during theme changes
function MdxWrapper({ code, customComponents = {} }: MdxWrapperProps) {
  const mergedComponents = {
    ...mdxComponents(),
    ...customComponents,
  } as MdxComponents;

  return <MDXContent code={code} components={mergedComponents} />;
}

// Export memoized version that only re-renders if code or components change
export default memo(MdxWrapper, (prevProps, nextProps) => {
  // Only re-render if the MDX code or custom components actually change
  return (
    prevProps.code === nextProps.code &&
    JSON.stringify(prevProps.customComponents) === JSON.stringify(nextProps.customComponents)
  );
});
