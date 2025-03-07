import React from 'react';
import ReactDOM from 'react-dom/client';
import Home from './app/Home.tsx';
import './App.css';
import { ThemeProvider } from '@/components/theme-provider.tsx';

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <ThemeProvider defaultTheme="dark">
      <Home />
    </ThemeProvider>
  </React.StrictMode>,
);
