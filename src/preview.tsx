import React from 'react';
import ReactDOM from 'react-dom/client';
import SearchBox from './components/SearchBox';

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
    <React.StrictMode>
        <main>
            <a href={import.meta.env.BASE_URL}>Back to home</a>
            <h1>Natal Chart Solver</h1>
            <SearchBox />
        </main>
    </React.StrictMode>
);
