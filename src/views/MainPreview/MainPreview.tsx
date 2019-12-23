import React, { FunctionComponent } from 'react';

import './MainPreview.scss';
import { selectModifiedImage, selectOriginalImage } from '../../store/selectors';
import { connect } from 'react-redux';
import { AppState } from '../../store/types';

interface MainPreviewProps {
  originalImage: string | null;
  modifiedImage: string | null;
}

const MainPreview: FunctionComponent<MainPreviewProps> = ({
                                                            originalImage,
                                                            modifiedImage
                                                          }) => {
  return <div className="m-main-preview">
    {(modifiedImage || originalImage) &&
    <img src={'data:image/png;base64,' + (modifiedImage || originalImage)} alt="Een voorbeeld van de aangepaste afbeelding"/>
    }
  </div>;
};

const mapStateToProps = (state: { app: AppState }) => ({
  originalImage: selectOriginalImage(state),
  modifiedImage: selectModifiedImage(state)
});

export default connect(mapStateToProps)(MainPreview);
