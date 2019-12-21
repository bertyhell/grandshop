import React, { FunctionComponent, useState } from 'react';
import './MainPreview.css';

interface MainPreviewProps {}

const MainPreview: FunctionComponent<MainPreviewProps> = () => {
	const [originalImageSource, setOriginalImageSource] = useState<string>('https://source.unsplash.com/random/500x300');

	return <div className="m-main-preview">
		<div className="a-original-preview-wrapper" style={{ backgroundImage: `url(${originalImageSource})` }}/>
	</div>;
};

export default MainPreview;
