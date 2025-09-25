import type { Metadata } from "next";
import "katex/dist/katex.min.css";
import { Geist_Mono } from "next/font/google";
import "./globals.css";
import Header from "./_components/ui/Header";
import Footer, { type FooterLink } from "./_components/ui/Footer";
import Starfield from "./_components/ui/Starfield";
import Script from "next/script";
import { cookies } from "next/headers";
import themeInitScript from "@/../scripts/init-theme.js";

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "Blog | aarondost.dev",
  description: "Blog by Aaron",
};

export default async function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  const cookieStore = await cookies();
  const themeCookie = cookieStore.get("theme")?.value;
  const initialTheme = themeCookie === "dark" ? "dark" : themeCookie === "light" ? "light" : undefined;
  const footerLinks: FooterLink[] = [
    {
      key: "about",
      displayText: "About",
      href: "https://www.aarondost.dev/about",
    },
    {
      key: "github",
      displayText: "GitHub",
      href: "https://github.com/Akaina1",
    },
    {
      key: "contact",
      displayText: "Contact",
      href: "https://www.aarondost.dev/contact",
    },
  ];
  return (
    <html suppressHydrationWarning className={initialTheme === "dark" ? "dark" : undefined}>
      <head>
        <link href="/favicon.ico" rel="icon" sizes="32x32" />
        <link href="/favicon.svg" rel="icon" type="image/svg+xml" />
        <Script id="theme-init" strategy="beforeInteractive">{themeInitScript}</Script>
      </head>
      <body className={`${geistMono.variable} antialiased`}>
        <Starfield />
        <Header />
        <main>{children}</main>
        <Footer links={footerLinks} />
      </body>
    </html>
  );
}
