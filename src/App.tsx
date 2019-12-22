import React from 'react';
import './App.scss';
import ResizablePanels from 'resizable-panels-react';
import MainPreview from './views/MainPreview/MainPreview';
import ControlsSidebar from './views/ControlsSidebar/ControlsSidebar';

const App: React.FC = () => {
  return (
      <div className="App">
        <ResizablePanels
            className="m-resizeable-panels"
            displayDirection="row"
            panelsSize={[60, 40]}
            sizeUnitMeasure="%"
            resizerSize="15px"
        >
          <MainPreview />
          <ControlsSidebar />
        </ResizablePanels>
      </div>
  );
};

export default App;
