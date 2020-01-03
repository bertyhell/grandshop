import React, { createRef, FunctionComponent, MutableRefObject, RefObject, useEffect, useRef } from 'react';
import 'viewerjs/dist/viewer.css';
import Viewer from 'viewerjs';

import './MainPreview.scss';
import { selectModificationInProgress, selectModifiedImage, selectOriginalImage } from '../../store/selectors';
import { connect } from 'react-redux';
import { AppState } from '../../store/types';
import Spinner from '../../components/Spinner/Spinner';

declare const ResizeObserver: any;

interface MainPreviewProps {
  originalImage: string | null;
  modifiedImage: string | null;
  modificationInProgress: boolean;
}

const MainPreview: FunctionComponent<MainPreviewProps> = ({
                                                            originalImage,
                                                            modifiedImage,
                                                            modificationInProgress
                                                          }) => {
  let el: RefObject<HTMLImageElement> = createRef<HTMLImageElement>();
  let viewer: MutableRefObject<any | null> = useRef<any | null>(null);

  const getImageSource = () => {
    return 'data:image/png;base64,' + (modifiedImage || originalImage);
  };

  const updateViewer = () => {
    if (viewer.current) {
      viewer.current.update();
    }
  };

  useEffect(() => { // eslint-disable-line react-hooks/exhaustive-deps
    if (!el.current || viewer.current) {
      return;
    }
    const DEFAULT_BUTTON = {
      show: 1,
      size: 'large' as any
    };
    viewer.current = new Viewer(el.current, {
      inline: true,
      backdrop: false,
      button: false,
      navbar: false,
      title: false,
      toolbar: {
        zoomIn: DEFAULT_BUTTON,
        zoomOut: DEFAULT_BUTTON,
        oneToOne: 0,
        reset: DEFAULT_BUTTON,
        prev: 0,
        play: 0,
        next: 0,
        rotateLeft: DEFAULT_BUTTON,
        rotateRight: DEFAULT_BUTTON,
        flipHorizontal: DEFAULT_BUTTON,
        flipVertical: DEFAULT_BUTTON
      },
      transition: false,
      fullscreen: false,
      movable: true,
      rotatable: true,
      scalable: true,
      zIndexInline: 1
    });

    // Listen for preview div size changes, so we can update the viewer
    new ResizeObserver(updateViewer).observe(el.current)
  }, [el]);

  useEffect(() => {
    console.log('updating viewer');
    updateViewer();
  }, [modifiedImage]); // eslint-disable-line react-hooks/exhaustive-deps

  return <div className="m-main-preview">
    {(modifiedImage || originalImage) &&
    <img className="o-image-viewer"
         ref={el}
         src={getImageSource()}
         alt="Een voorbeeld van de aangepaste afbeelding"/>
    }
    {modificationInProgress &&
    <div className="o-spinner-wrapper">
      <Spinner/>
      Afbeelding wordt geupdate...
    </div>
    }
  </div>;
};

const mapStateToProps = (state: { app: AppState }) => ({
  originalImage: selectOriginalImage(state),
  modifiedImage: selectModifiedImage(state),
  modificationInProgress: selectModificationInProgress(state)
});

export default connect(mapStateToProps)(MainPreview);
