import os
import bson
import json
from datetime import datetime
from typing import TypeAlias, Union



strOrList: TypeAlias = Union[str, list]


class Log:

    def __init__(self, db="", db_path=None):
        """
        Initiate instances of the db.ox-db

        Args:
            db (str, optional): The name of the db or path/name that gets accessed or instantiated. Defaults to "".

        Returns:
            None
        """
        self.set_db(db, db_path)
        self.doc = None
        self.set_doc()
  

    def set_db(self, db, db_path=None):
        self.db = db
        self.db_path = (
            db_path if db_path else os.path.join(os.path.expanduser("~"), db + ".ox-db")
        )
        os.makedirs(self.db_path, exist_ok=True)  # Create directory if it doesn't exist

        return db

    def get_db(self):
        return self.db_path

    def set_doc(self, doc=None, doc_format="bson"):
        if doc is None:
            self.doc = self.doc or "log-" + datetime.now().strftime("[%d_%m_%Y]")
        elif self.doc:
            if self.doc == doc:
                return self.doc
            else:
                self.doc = doc

        self.doc_path = os.path.join(self.db_path, self.doc)
        os.makedirs(self.doc_path, exist_ok=True)
        if doc_format not in ["bson", "json"]:
            raise ValueError("doc_format must be 'bson' or 'json'")
        self.doc_format = doc_format

        file_content = self.load_data(self.doc + ".index")
        self.doc_entry = file_content["ox-db_init"]["doc_entry"]

        return self.doc

    def get_doc(self):
        return self.doc or self.set_doc()

    def push(
        self,
        data: strOrList,
        embeddings: strOrList = None,
        data_story: strOrList = None,
        key: strOrList = None,
        doc: strOrList = None,
    ):
        """
        Pushes data to the log file. Can be called with either data or both key and data.

        Args:
            data (any, optional): The data to be logged.
            key (str, optional): The key for the log entry. Defaults to eg: ("04-06-2024") current_date
            doc (str, optional): The doc for the log entry. Defaults to eg: ("10:30:00-AM") current_time with AM/PM

        Returns:
            None
        """
        doc = self.set_doc(doc) if doc else self.get_doc()
        uid = self.gen_uid(key)

        if data == "" or data == None:
            raise ValueError("ox-db : no prompt is given")
        if not embeddings:
            embeddings = data

        index_data = {
            "uid": uid,
            "key": key or "key",
            "doc": doc,
            "time": datetime.now().strftime("%I:%M:%S_%p"),
            "date": datetime.now().strftime("%d_%m_%Y"),
            "vec_model": "vec.model",
            "discription": data_story,
        }

        self._push(uid, index_data, doc + ".index")
        self._push(uid, data, doc)
        self._push(uid, embeddings, doc + ".ox-vec")

        return uid

    def pull(
        self,
        uid: strOrList = None,
        key: str = None,
        time: str = None,
        date: str = None,
        doc: str = None,
        docfile:str = None
    ):
        """
        Retrieves a specific log entry from a BSON or JSON file based on date and time.

        Args:
            key (any or optional): datakey or The time of the log entry in the format used by push eg: ("10:30:00-AM").
            doc (any or optional): doc or date of the log entry in the format used by push eg: ("04-06-2024").

        Returns:
            any: The log data associated with the specified key,time and doc,date or None if not found.
        """
        all_none = all(var is None for var in [key, uid, time, date])

        doc = doc or (self.doc or "log-" + datetime.now().strftime("[%d_%m_%Y]"))

        docfile = docfile or doc
        log_entries = []
        if all_none:
            content = self.load_data(docfile)
            for uidx, data in content.items():
                if uidx == "ox-db_init":
                    continue
                log_entries.append(
                    {
                        "uid": uidx,
                        "data": data,
                    }
                )
            return log_entries

        

        if uid is not None:
            [uids] = Log._convert_input(uid)
            content = self.load_data(docfile)
            for uid in uids:
                if uid in content:
                    data = content[uid]
                    log_entries.append(
                        {
                            "uid": uid,
                            "data": data,
                        }
                    )
            return log_entries

        if any([key, time, date]):
            uids = self.search_uid(doc, key, time, date)
            data = self.pull(uid=uids,doc=doc,docfile=docfile)
            log_entries.extend(data)
            return log_entries



    def gen_uid(self, key=None):

        key = key or "key"

        uid = (
            str(self.doc_entry)
            + "-"
            + key
            + "-"
            + do.generate_random_string()
        )

        return uid

    def load_data(self, log_file):
        log_file_path = self._get_logfile_path(log_file)
        try:
            with open(
                log_file_path, "rb+" if self.doc_format == "bson" else "r+"
            ) as file:
                if self.doc_format == "bson":
                    file_content = file.read()
                    return bson.decode_all(file_content)[0] if file_content else {}
                else:
                    is_empty = file.tell() == 0
                    return json.load(file) if is_empty else {}
        except FileNotFoundError:
            file_content = {"ox-db_init": {"doc_entry": 0}}
            self.save_data(log_file, file_content)
            return file_content

    def save_data(self, log_file, file_content):
        log_file_path = self._get_logfile_path(log_file)

        def write_file(file, content, format):
            file.seek(0)
            file.truncate()
            if format == "bson":
                file.write(bson.encode(content))
            else:
                json.dump(content, file, indent=4)

        try:
            mode = "rb+" if self.doc_format == "bson" else "r+"
            with open(log_file_path, mode) as file:
                write_file(file, file_content, self.doc_format)
        except FileNotFoundError:
            mode = "wb" if self.doc_format == "bson" else "w"
            with open(log_file_path, mode) as file:
                write_file(file, file_content, self.doc_format)

    def search_uid(self, doc=None, key=None, time=None, date=None):
        doc = doc or (self.doc or "log-" + datetime.now().strftime("[%d_%m_%Y]"))
        content = self.load_data(doc + ".index")
        uids = []
        itime_parts = [None, None, None, None]
        idate_parts = [None,None,None,]

        if time:
            itime, ip = (
                time.split("_")
                if "_" in time
                else (time, datetime.now().strftime("%p"))
            )
            itime_parts = itime.split(":") + [ip]

        if date:
            idate_parts = date.split("_") if "_" in date else [date]

        for uid, data in content.items():
            if uid == "ox-db_init":
                continue

            log_it = False
            data["time"]
            time_parts = data["time"].split("_")[0].split(":") + [
                data["time"].split("_")[1]
            ]
            date_parts = data["date"].split("_")

            if time_parts[: len(itime_parts) - 1] == itime_parts[:-1]:
                if time_parts[-1] == itime_parts[-1]:
                    log_it = True

            elif date_parts[: len(idate_parts)] == idate_parts:
                log_it = True

            elif key == data["key"]:
                log_it = True

            else:
                log_it = False

            if log_it:
                uids.append(uid)

        return uids


    def _get_logfile_path(self, log_file):

        self.doc_path = self.doc_path or os.path.join(self.db_path, self.doc)
        logfile_path = os.path.join(self.doc_path, f"{log_file}.{self.doc_format}")
        return logfile_path

    def _push(self, uid, data, log_file):

        if data == "" or data == None:
            raise ValueError("ox-db : no prompt is given")

        file_content = self.load_data(log_file)
        file_content[uid] = data
        if "." in log_file:
            if log_file.split(".")[1] == "index":
                file_content["ox-db_init"]["doc_entry"] += 1
                self.doc_entry = file_content["ox-db_init"]["doc_entry"]
        self.save_data(log_file, file_content)

        print(f"ox-db : logged data : {uid} \n{log_file}")


    @classmethod
    def _convert_input(cls,*args: Union[str, list]) -> list:
        """
        Converts input arguments (any number) to lists if they are strings.
        """

        converted_args = []
        for arg in args:
            if isinstance(arg, str):
                converted_args.append([arg])
            elif arg == None:
                converted_args.append([])
            else:
                converted_args.append(arg)

        return converted_args