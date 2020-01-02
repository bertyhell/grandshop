import React, { FunctionComponent } from 'react';

import './MainPreview.scss';
import { selectModificationInProgress, selectModifiedImage, selectOriginalImage } from '../../store/selectors';
import { connect } from 'react-redux';
import { AppState } from '../../store/types';
import Spinner from '../../components/Spinner/Spinner';

interface MainPreviewProps {
  originalImage: string | null;
  modifiedImage: string | null;
  modificationInProgress: boolean;
}

const MainPreview: FunctionComponent<MainPreviewProps> = ({
                                                            originalImage,
                                                            modifiedImage,
                                                            modificationInProgress,
                                                          }) => {
  return <div className="m-main-preview">
    {(modifiedImage || originalImage) &&
    <img src={'data:image/png;base64,' + (modifiedImage || originalImage)} alt="Een voorbeeld van de aangepaste afbeelding"/>
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
  modificationInProgress: selectModificationInProgress(state),
});

export default connect(mapStateToProps)(MainPreview);
