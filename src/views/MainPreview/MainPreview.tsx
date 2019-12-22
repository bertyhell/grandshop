import React, { FunctionComponent } from 'react';

import './MainPreview.scss';
import { selectModifiedImageUrl, selectOriginalImageUrl } from '../../store/selectors';
import { connect } from 'react-redux';
import { AppState } from '../../store/types';

interface MainPreviewProps {
  originalImageUrl: string | null;
  modifiedImageUrl: string | null;
}

const MainPreview: FunctionComponent<MainPreviewProps> = ({
                                                            originalImageUrl,
                                                            modifiedImageUrl
                                                          }) => {
  return <div className="m-main-preview">
    {(modifiedImageUrl || originalImageUrl) &&
    <div
      className="a-preview-wrapper"
      style={{ backgroundImage: `url('file://${modifiedImageUrl || originalImageUrl}')` }}
    />
    }
  </div>;
};

const mapStateToProps = (state: { app: AppState }) => ({
  originalImageUrl: selectOriginalImageUrl(state),
  modifiedImageUrl: selectModifiedImageUrl(state)
});

export default connect(mapStateToProps)(MainPreview);
