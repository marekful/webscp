#!/usr/bin/with-contenv bash

ARCHIVE_NANE=$1
ARCHIVE_PATH=/app/data/temp/$ARCHIVE_NANE.dst.tar
EXTRACT_PATH=$2
IS_COMPRESSED=$3
OVERWRITE=$4

SKIP_OLD_FILES_FLAG=
GZIP_FLAG=

if [ "$IS_COMPRESSED" = "true" ]; then
  GZIP_FLAG=-z
fi

if [ "$DISTRO" = "debian" ]; then

  if [ "$OVERWRITE" = "false" ]; then
    SKIP_OLD_FILES_FLAG=--skip-old-files
  fi

  tar $SKIP_OLD_FILES_FLAG -x $GZIP_FLAG -f $ARCHIVE_PATH -C $EXTRACT_PATH

  rm -rf ARCHIVE_PATH

elif [ "$DISTRO" = "alpine" ]; then

  if [ "$OVERWRITE" = "true" ]; then
    tar -x $GZIP_FLAG -f $ARCHIVE_PATH -C $EXTRACT_PATH
  else
    TMP_PATH=/app/data/temp/$ARCHIVE_NANE-tmp
    #echo $TMP_PATH
    mkdir $TMP_PATH
    #echo tar -x $GZIP_FLAG -f $ARCHIVE_PATH -C $TMP_PATH
    tar -x $GZIP_FLAG -f $ARCHIVE_PATH -C $TMP_PATH
    #echo rsync -r --ignore-existing $TMP_PATH/* $EXTRACT_PATH
    rsync -r --ignore-existing $TMP_PATH/* $EXTRACT_PATH
    rm -rf $TMP_PATH
  fi

  rm -rf ARCHIVE_PATH

fi
