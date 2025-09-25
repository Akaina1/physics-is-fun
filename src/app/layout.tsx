import type { Metadata } from "next";
import "katex/dist/katex.min.css";
import { Geist_Mono } from "next/font/google";
import "./globals.css";
import Header from "./_components/ui/Header";
import Footer, { type FooterLink } from "./_components/ui/Footer";
import Starfield from "./_components/ui/Starfield";

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "Blog | aarondost.dev",
  description: "Blog by Aaron",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
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
    <html>
      <head>
        <link href="/favicon.ico" rel="icon" sizes="32x32" />
        <link href="/favicon.svg" rel="icon" type="image/svg+xml" />
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
