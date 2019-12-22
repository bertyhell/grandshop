import React, { FunctionComponent, useState } from 'react';
import './ControlsSidebar.css';
import { transformImage } from '../../electron-helpers/interop-web';

interface ControlsSidebarProps {
}

const ControlsSidebar: FunctionComponent<ControlsSidebarProps> = () => {
  const [thickness, setThickness] = useState<string>('30');
  const [color, setColor] = useState<string>('#225eff');

  const applyBorder = () => {
    try {
      transformImage(parseInt(thickness, 10), color);
    } catch (err) {
      // show toast
    }
  };

  return (
      <div className="m-controls-sidebar">
        <div>
          <h2>Rand toevoegen</h2>
          <div className="form-field">
            <label>Dikte:</label>
            <input type="text" value={thickness} onChange={(evt) => {
              console.log('thickness changed: ', evt);
              setThickness(evt.target.value)
            }}/>
          </div>
          <div className="form-field">
            <label>Kleur:</label>
            <input type="color" value={color} onChange={function() { setColor((this as any).value) }}/>
          </div>
          <button id="apply-border-button" onClick={applyBorder}>Rand toepassen</button>
        </div>
      </div>
  );
};

export default ControlsSidebar;
