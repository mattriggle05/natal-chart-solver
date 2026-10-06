import { useState } from 'react';
import type { FormEvent } from 'react';
import { useDataSearch } from '../hooks/useDateSearch';
import { Feature } from '../types/features';
import { Signs } from '../types/signs';

const bodies = [Feature.Sun, Feature.Moon, Feature.Mercury, Feature.Venus, Feature.Mars, Feature.Jupiter, Feature.Saturn, Feature.Uranus, Feature.Neptune];
const signs = Object.values(Signs).filter((value): value is string => typeof value === 'string');

export default function SearchBox() {
    const [placements, setPlacements] = useState<Record<number, string>>({});
    const [startDate, setStartDate] = useState('1900-01-01');
    const [endDate, setEndDate] = useState('2100-01-01');
    const [error, setError] = useState('');
    const { search, cancel, busy, output } = useDataSearch();

    function submit(event: FormEvent<HTMLFormElement>) {
        event.preventDefault();
        const selected = bodies.filter(body => placements[body] !== undefined && placements[body] !== '');
        if (!selected.length) { setError('Select at least one sign.'); return; }
        if (startDate >= endDate) { setError('End date must be after start date.'); return; }
        setError('');
        search({ startDate, endDate, featureIds: selected, angleStarts: selected.map(body => Number(placements[body]) * 30), angleSpans: selected.map(() => 30) });
    }

    return <>
        <form onSubmit={submit}>
            <fieldset disabled={busy}>
                <legend>Signs</legend>
                {bodies.map(body => <div key={body}>
                    <label htmlFor={`body-${body}`}>{Feature[body]} </label>
                    <select id={`body-${body}`} value={placements[body] ?? ''} onChange={event => setPlacements({ ...placements, [body]: event.target.value })}>
                        <option value="">Any sign</option>
                        {signs.map((sign, index) => <option key={sign} value={index}>{sign}</option>)}
                    </select>
                </div>)}
                <p><label>Start date <input type="date" required min="1900-01-01" max="2099-12-31" value={startDate} onChange={event => setStartDate(event.target.value)} /></label></p>
                <p><label>End date (exclusive) <input type="date" required min="1900-01-02" max="2100-01-01" value={endDate} onChange={event => setEndDate(event.target.value)} /></label></p>
                <button type="submit">Search</button>
            </fieldset>
            {busy && <button type="button" onClick={cancel}>Cancel</button>}
        </form>
        <p>Dates use UTC (estimated UT before 1972). Searches at 1900 and 2100 are limited to the supported range.</p>
        <p>Times are rounded to the nearest minute. Astronomical boundaries are approximate.</p>
        {error && <p role="alert">{error}</p>}
        <h2>Output</h2>
        <div role="status" aria-live="polite">{output.split('\n').map((line, index) => <div key={index}>{line}</div>)}</div>
    </>;
}
