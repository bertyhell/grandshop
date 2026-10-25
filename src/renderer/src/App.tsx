import React from 'react';
import { ToastContainer } from 'react-toastify';
import { Allotment } from 'allotment';
import 'allotment/dist/style.css';
import { Provider } from 'react-redux';

import MainPreview from './views/MainPreview/MainPreview';
import ControlsSidebar from './views/ControlsSidebar/ControlsSidebar';
import store from './store/store';

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
				<Allotment
					onChange={() => {
						window.dispatchEvent(new Event('resize'));
					}}
					className="m-resizeable-panels"
					defaultSizes={[60, 40]}
				>
					<MainPreview />
					<ControlsSidebar />
				</Allotment>
			</div>
		</Provider>
	);
};

export default App;
