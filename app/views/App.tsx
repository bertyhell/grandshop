import { hot } from 'react-hot-loader/root';
import React from 'react';
import { ToastContainer } from 'react-toastify';
import ResizablePanels from 'resizable-panels-react';
import { Provider } from 'react-redux';

import MainPreview from './MainPreview/MainPreview';
import ControlsSidebar from './ControlsSidebar/ControlsSidebar';
import store from '../store/store';

import './Global.scss';
import './App.scss';

const App: React.FC = () => {
  return (
      <Provider store={store}>
        <div className="m-app">
          <ToastContainer
              className="c-alert-stack"
              closeButton={false}
              closeOnClick={false}
              draggable={false}
              position="top-left"
          />
          <ResizablePanels
              className="m-resizeable-panels"
              displayDirection="row"
              panelsSize={[60, 40]}
              sizeUnitMeasure="%"
              resizerSize="15px"
              onResize={() => {
                window.dispatchEvent(new Event('resize'));
              }}
          >
            <MainPreview/>
            <ControlsSidebar/>
          </ResizablePanels>
        </div>
      </Provider>
  );
};

export default hot(App);
