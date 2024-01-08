import type { Metadata } from 'next'
import { Inter } from 'next/font/google'
import { ThemeProvider } from "@/components/theme-provider"
import './globals.css'
import TitleBar from '@/components/titlebar'
import { Toaster } from "@/components/ui/toaster"
import ConfigsProvider from "../components/context/ConfigContext";


const inter = Inter({ subsets: ['latin'] })

export const metadata: Metadata = {
  title: 'Mp3 Automated Tag Editor',
  description: 'Automated Music Metadata Generator',
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {

  const defaultData = {
    "test": "test",
    "threads": 1,
    "developerSettings": false,
}

  return (
    <html lang="en" className='scroll-smooth' style={{scrollBehavior:'smooth'}}>
      {/* <TitleBar titleText={""}/> */}
      <ThemeProvider
        attribute="class"
        defaultTheme="system"
        enableSystem
        disableTransitionOnChange
      >
        <TitleBar titleBar='Automated Mp3 Tag Editor - v1.0' />
        <ConfigsProvider defaultData={defaultData}>

          <body className={inter.className}>
            {children}
          </body>
        </ConfigsProvider>

        <Toaster />
      </ThemeProvider>
    </html>
  )
}
