import React, { FunctionComponent, useState } from 'react';
import'./LeftSidebar.css';
import { openFileDialog } from 'electron-helpers/interop';

interface LeftSidebarProps {
}

const LeftSidebar: FunctionComponent<LeftSidebarProps> = () => {
  const [originalImageSource, setOriginalImageSource] = useState<string>('https://source.unsplash.com/random/500x300');

  return (
      <div className="m-left-sidebar">
        <button id="open-image-button" onClick={() => openFileDialog()}>
          Open afbeelding
        </button>
        <div className="a-original-preview-wrapper" style={{ backgroundImage: `url(${originalImageSource})` }}/>
      </div>
  );
};

export default LeftSidebar;
