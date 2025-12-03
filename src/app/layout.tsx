import type { ReactNode } from 'react';
import './globals.css';

export const metadata = {
  title: 'Creator App',
  description: 'Baseline app with telemetry and multi-tenant scaffolding'
};

type Props = {
  children: ReactNode;
};

export default function RootLayout({ children }: Props) {
  return (
    <html lang="en">
      <body className="min-h-screen bg-white text-slate-900 antialiased">
        <main className="mx-auto max-w-5xl px-4 py-8">{children}</main>
      </body>
    </html>
  );
}
