import React from 'react';
import ResizablePanels from 'resizable-panels-react';
import './App.css';
import ControlsSidebar from './components/ControlsSidebar/ControlsSidebar';
import LeftSidebar from './components/LeftSidebar/LeftSidebar';
import MainPreview from './components/MainPreview/MainPreview';

function App() {
	return (
		<div className="App">
			<ResizablePanels
				displayDirection="row"
				panelsSize={[20, 50, 30]}
				sizeUnitMeasure="%"
				resizerColor="#353b48"
				resizerSize="15px"
			>
				<LeftSidebar />
				<MainPreview />
				<ControlsSidebar />
			</ResizablePanels>
		</div>
	);
}

export default App;
