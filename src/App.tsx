import React from 'react';
import './App.scss';
import ResizablePanels from 'resizable-panels-react';
import MainPreview from './views/MainPreview/MainPreview';
import ControlsSidebar from './views/ControlsSidebar/ControlsSidebar';
import { Provider } from 'react-redux';
import store from './store/store';

const App: React.FC = () => {
  return (
      <Provider store={store}>
        <div className="App">
          <ResizablePanels
              className="m-resizeable-panels"
              displayDirection="row"
              panelsSize={[60, 40]}
              sizeUnitMeasure="%"
              resizerSize="15px"
          >
            <MainPreview/>
            <ControlsSidebar/>
          </ResizablePanels>
        </div>
      </Provider>
  );
};

export default App;
