import React, { FunctionComponent, useEffect, useReducer, useState } from 'react';
import getUuid from 'uuid/v1';
import { connect } from 'react-redux';
import { useDebounce } from 'use-debounce';
import { Dispatch } from 'redux';
import usePrevious from 'use-previous';
import compact from 'lodash-es/compact';
import map from 'lodash-es/map';
import kebabCase from 'lodash-es/kebabCase';
import fromPairs from 'lodash-es/fromPairs';
import throttle from '@jcoreio/async-throttle';

import { applyModifications, saveImage, openImage } from '../../services/interop-electron';
import Button from '../../components/Button/Button';
import { AppState } from '../../store/types';
import { selectModifiedImage, selectOriginalImage } from '../../store/selectors';
import { setModificationInProgressAction, setModifiedImageAction } from '../../store/actions';
import { Select, SelectOption } from '../../components/Select/Select';
import { format } from '../../helpers/format';
import mods, { ModificationComponent } from '../../shared/modifications/components';
import { ImageModificationInfo } from '../../shared/modifications/types';
import ImageModification from '../../components/ImageModification/ImageModification';
import { ImageInfo } from '../../shared/types';

import './ControlsSidebar.scss';
import { isEqual } from 'lodash-es';

interface ControlsSidebarProps {
  originalImage: ImageInfo | null;
  modifiedImage: ImageInfo | null;
  setModificationInProgress: (inProgress: boolean) => void;
  setModifiedImage: (image: ImageInfo | null) => void;
}

interface ModifierState {
  openStates: boolean[];
  activeModifiers: ActiveModifier[];
}

interface ActiveModifier {
  name: string;
  title: string;
  info: ImageModificationInfo | null;
  key: string;
}

const ControlsSidebar: FunctionComponent<ControlsSidebarProps> = ({
                                                                    originalImage,
                                                                    modifiedImage,
                                                                    setModificationInProgress,
                                                                    setModifiedImage,
                                                                  }) => {
  const [selectedOption, setSelectedOption] = useState<string | null>(null);

  const IMAGE_MODIFIER_OPTIONS: SelectOption[] = [
    ...map(mods, (mod: ModificationComponent, modName: string) => {
      return {
        label: mod.label,
        value: modName
      }
    })
  ];

  const IMAGE_MODIFIER_TITLES: { [modifierName: string]: string } = fromPairs(map(mods, (mod: ModificationComponent, modName: string) => {
    return [modName, mod.title];
  }));

  type ActiveModifierAction =
      | { type: 'ADD', name: string }
      | { type: 'UPDATE', info: ImageModificationInfo, index: number }
      | { type: 'TOGGLE', index: number }
      | { type: 'DELETE', index: number }

  const getModifierTitle = (activeModifiers: ActiveModifier[], modifierName: string): string => {
    return format(IMAGE_MODIFIER_TITLES[modifierName], activeModifiers.filter(mod => mod.name === modifierName).length + 1);
  };

  function activeModifiersReducer(modifierState: ModifierState, action: ActiveModifierAction): ModifierState {
    let newModifiers: ActiveModifier[] | undefined = [...modifierState.activeModifiers];
    let newOpenStates: boolean[] | undefined = [...modifierState.openStates];

    let key = getUuid();
    switch (action.type) {
      case 'ADD':
        const title: string = getModifierTitle(modifierState.activeModifiers, action.name);
        newModifiers.push({
          title,
          info: null,
          name: action.name,
          key
        });
        newOpenStates =  [
          ...modifierState.openStates.map(c => false), // Collapse all other modifiers when add a new one
          true
        ];
        break;

      case 'UPDATE':
        newModifiers = [...modifierState.activeModifiers];
        newModifiers[action.index] = {
          ...newModifiers[action.index],
          info: action.info
        };
        break;

      case 'TOGGLE':
        newOpenStates[action.index] = !newOpenStates[action.index];
        break;

      case 'DELETE':
        newModifiers.splice(action.index, 1);
        newOpenStates.splice(action.index, 1);
        break;
    }

    return {
      activeModifiers: newModifiers,
      openStates: newOpenStates
    }
  }

  const [modifierState, changeModifierState] = useReducer<React.Reducer<ModifierState, ActiveModifierAction>>(activeModifiersReducer, {
    openStates: [],
    activeModifiers: []
  });
  const previousActiveModifiers = usePrevious<ActiveModifier[] | null>(modifierState.activeModifiers);
  const applyModificationsThrottled = throttle((modifications: ImageModificationInfo[]) => {
    applyModifications(modifications);
  }, 0);

  useEffect(() => {
    // check previous state to see if we need to run the applyModifications function because all modifiers were removed
    if (!modifierState.activeModifiers || !modifierState.activeModifiers.length) {
      // No modifications, remove modified image, so the original image will be shown
      setModifiedImage(null);
    } else if (!isEqual(previousActiveModifiers, modifierState.activeModifiers)) {
      // Send the modifications to nodejs so we can update the image
      setModificationInProgress(true);
      const modifications: ImageModificationInfo[] = compact(modifierState.activeModifiers.map((m: ActiveModifier) => m.info));
      applyModificationsThrottled(modifications); // Do not apply new modifications if the previous modifications are not done yet
    }
  }, [modifierState.activeModifiers]); // eslint-disable-line react-hooks/exhaustive-deps

  // Reset the selected dropdown option back to the placeholder, to users know they can add another modifier
  useEffect(() => {
    if (selectedOption !== '') {
      setSelectedOption('');
    }
  }, [selectedOption]);

  const handleApplyModifier = (modifierInfo: ImageModificationInfo, index: number) => {
    changeModifierState({ type: 'UPDATE', info: modifierInfo, index });
  };

  const handleToggleModifier = (index: number) => {
    changeModifierState({ type: 'TOGGLE', index });
  };

  const handleDeleteModifier = (index: number) => {
    changeModifierState({ type: 'DELETE', index });
  };

  const handleModifierSelectionChanged = (name: string) => {
    changeModifierState({ type: 'ADD', name });
    setSelectedOption(name);
  };

  return (
      <div className="m-controls-sidebar">
        <div className="m-button-wrapper">
          <Button onClick={() => {
            openImage({
              buttonLabel: "Openen",
              // defaultPath // TODO default to the users images folder
              filters: [
                {
                  name: 'Alle bestanden',
                  extensions: ['*']
                },
                {
                  name: 'Afbeeldingen',
                  extensions: ['jpg', 'png', 'gif', 'tif', 'bmp', 'jpeg', 'webp']
                }
              ],
              title: 'Selecteer de afbeelding die je wil bewerken',
              properties: [
                // 'multiSelections', // TODO add multi image support
                'openFile'
              ]
            });
          }} label="Afbeelding openen"/>
          {modifiedImage &&
          <Button className="c-save-button" onClick={() => {
            saveImage({
              buttonLabel: "Opslaan",
              // defaultPath // TODO default to the users images folder
              filters: [
                {
                  name: 'Alle bestanden',
                  extensions: ['*']
                },
                {
                  name: 'Afbeeldingen',
                  extensions: ['jpg', 'png', 'gif', 'tif', 'bmp', 'jpeg', 'webp']
                }
              ],
              title: 'Kies de locatie en naam om je afbeelding op te slaan'
            });
          }} label="Afbeelding opslaan als"/>
          }
        </div>
        {originalImage &&
        <>
          <div className="m-button-wrapper">
            <Select
              options={IMAGE_MODIFIER_OPTIONS}
              onChange={handleModifierSelectionChanged}
              value={null}
              placeholder={modifierState.activeModifiers.length ? 'Kies nog een bewerking' : 'Kies een bewerking'}
            />
          </div>
          <div className="m-modification-controls">
            {modifierState.activeModifiers.map((m: ActiveModifier, index: number) => {
              if (!mods[m.name] || !mods[m.name].component) {
                console.error('Failed to find component by name in list of modifications', { modificationInfo: m });
                return null;
              }
              const Component = mods[m.name].component;
              return <ImageModification
                  label={m.title}
                  className={'c-' + kebabCase(m.name)}
                  isOpen={modifierState.openStates[index]}
                  onToggle={() => handleToggleModifier(index)}
                  onDelete={() => handleDeleteModifier(index)}
                  key={m.key}
              >
                <Component
                    imageInfo={modifiedImage || originalImage}
                    onApply={(info: ImageModificationInfo) => handleApplyModifier(info, index)}
                />
              </ImageModification>
            })}
          </div>
        </>
        }
      </div>
  );
};

const mapStateToProps = (state: { app: AppState }) => ({
  originalImage: selectOriginalImage(state),
  modifiedImage: selectModifiedImage(state)
});

const mapDispatchToProps = (dispatch: Dispatch) => {
  return {
    setModificationInProgress: (inProgress: boolean) => dispatch(setModificationInProgressAction(inProgress) as any),
    setModifiedImage: (image: ImageInfo | null) => dispatch(setModifiedImageAction(image) as any)
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(ControlsSidebar);
